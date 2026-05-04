const $ = (id) => document.getElementById(id);

const history = {
  cpu: Array(60).fill(0),
  gpu: Array(60).fill(0),
  down: Array(60).fill(0),
  up: Array(60).fill(0),
};

let paused = false;
let timer = null;
let processTab = "cpu";
let latestProcesses = {};

function setText(id, value) {
  const node = $(id);
  if (node) node.textContent = value;
}

function setWidth(id, value) {
  const node = $(id);
  if (node) node.style.width = `${Math.max(0, Math.min(100, value))}%`;
}

function setGauge(id, value) {
  const gauge = $(id);
  if (!gauge) return;
  const fill = gauge.querySelector(".fill");
  const percent = Math.max(0, Math.min(100, value));
  fill.style.strokeDashoffset = `${188 - (188 * percent) / 100}`;
}

function push(series, value) {
  series.push(Number.isFinite(value) ? value : 0);
  while (series.length > 60) series.shift();
}

function drawChart(id, points, color, maxValue = 100) {
  const canvas = $(id);
  if (!canvas) return;
  const ratio = window.devicePixelRatio || 1;
  const width = canvas.clientWidth;
  const height = canvas.clientHeight || Number(canvas.getAttribute("height")) || 100;
  canvas.width = width * ratio;
  canvas.height = height * ratio;
  const ctx = canvas.getContext("2d");
  ctx.scale(ratio, ratio);
  ctx.clearRect(0, 0, width, height);

  ctx.strokeStyle = "rgba(74, 63, 45, 0.1)";
  ctx.lineWidth = 1;
  ctx.font = "12px -apple-system, BlinkMacSystemFont, sans-serif";
  ctx.fillStyle = "#777267";
  for (let i = 0; i <= 4; i += 1) {
    const y = (height - 22) * (i / 4) + 4;
    ctx.beginPath();
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
    ctx.stroke();
  }

  const chartHeight = height - 26;
  const step = width / Math.max(1, points.length - 1);
  ctx.strokeStyle = color;
  ctx.lineWidth = 2;
  ctx.beginPath();
  points.forEach((value, index) => {
    const x = index * step;
    const y = chartHeight - (Math.min(value, maxValue) / maxValue) * (chartHeight - 6) + 4;
    if (index === 0) ctx.moveTo(x, y);
    else ctx.lineTo(x, y);
  });
  ctx.stroke();
}

function drawNetChart(id, points, color) {
  const max = Math.max(1024, ...points) * 1.2;
  drawChart(id, points, color, max);
}

function formatTime(timestamp) {
  return new Date(timestamp * 1000).toLocaleTimeString("fr-FR", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
}

function renderProcessList(items, metric) {
  const list = $("process-list");
  if (!list) return;
  list.replaceChildren();
  const rows = (items || []).slice(0, 10);
  if (!rows.length) {
    const empty = document.createElement("li");
    empty.innerHTML = `<span class="process-rank">--</span><span class="process-name">Aucune donnée</span><span class="process-metric">--</span>`;
    list.append(empty);
    return;
  }

  rows.forEach((item, index) => {
    const row = document.createElement("li");
    const rawImpact = metric === "ram" ? memoryToMb(item.memory) / 1024 : metric === "heat" ? item.heat : item.cpu;
    const impact = Math.max(6, Math.min(100, metric === "ram" ? rawImpact * 16 : rawImpact));
    const value = metric === "ram" ? item.memory : metric === "heat" ? `${Math.round(item.heat)} score` : `${item.cpu.toFixed(1)}%`;
    const impactLabel = metric === "ram" ? "RAM" : metric === "heat" ? "Thermique" : processTab === "rosetta" ? "Rosetta" : "CPU";
    const badge = item.badge ? `<span class="process-badge"> · ${escapeHtml(item.badge)}</span>` : "";
    const subject = item.processCount > 1 ? `${item.processCount} proc.` : `PID ${item.pid}`;
    const windowLabel = item.windowSeconds >= 285 ? "5 min" : `${Math.max(1, Math.round(item.windowSeconds / 60))} min`;
    const arch = item.architecture && item.architecture !== "unknown" ? ` · ${item.architecture}` : "";
    const cause = item.cause || "Processus direct de l'app ou service macOS standard.";
    row.style.setProperty("--impact", `${impact}%`);
    row.className = metric === "ram" ? "process-memory" : metric === "heat" ? "process-heat" : "process-cpu";
    row.innerHTML = `
      <span class="process-rank">${index + 1}</span>
      <span class="process-name" title="${escapeHtml(item.name)}">${escapeHtml(item.name)}</span>
      <span class="process-metric">${value}</span>
      <span class="process-meta">${subject} · ${item.memory} · CPU ${item.cpu.toFixed(1)}%${arch}</span>
      <span class="process-hint">moy. ${windowLabel} · ${escapeHtml(item.hint)}${badge}</span>
      <span class="process-cause" title="${escapeHtml(cause)}">${impactLabel} · ${escapeHtml(cause)}</span>
    `;
    list.append(row);
  });
}

function memoryToMb(value) {
  const text = String(value ?? "").trim();
  const match = text.match(/^([\d.]+)\s*([KMGT]?B)$/i);
  if (!match) return 0;
  const amount = Number(match[1]);
  const unit = match[2].toUpperCase();
  const factor = unit === "TB" ? 1024 * 1024 : unit === "GB" ? 1024 : unit === "KB" ? 1 / 1024 : 1;
  return Number.isFinite(amount) ? amount * factor : 0;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function translateHealth(value) {
  return value === "Good" ? "Bon" : value === "Check" ? "À vérifier" : value;
}

function translateWarning(value) {
  const map = {
    "CPU high": "CPU élevé",
    "Memory pressure high": "Pression mémoire élevée",
    "Disk nearly full": "Disque presque plein",
    "Battery condition": "Batterie à vérifier",
    "Process CPU high": "Processus CPU élevé",
  };
  return map[value] || value;
}

function updateProcesses(processes = {}) {
  latestProcesses = processes;
  const topCpu = processes.topCpu || [];
  const topMemory = processes.topMemory || [];
  const topHeat = processes.topHeat || [];
  const sleepers = processes.sleepers || [];
  const rosetta = processes.rosetta || {};
  const rosettaSuspects = rosetta.suspects || [];
  const cpuLeader = topCpu[0];
  const ramLeader = topMemory[0];
  const heatLeader = topHeat[0];
  const rosettaLeader = rosettaSuspects[0];

  setText("cpu-leader", cpuLeader ? `${cpuLeader.name} · ${cpuLeader.cpu.toFixed(1)}%` : "--");
  setText("ram-leader", ramLeader ? `${ramLeader.name} · ${ramLeader.memory}` : "--");
  setText("heat-leader", heatLeader ? `${heatLeader.name} · ${Math.round(heatLeader.heat)} score` : "--");
  setText("rosetta-leader", rosettaLeader ? `${rosettaLeader.name} · Intel` : rosetta.active ? "oahd actif" : "aucun suspect");
  setText("process-alert", cpuLeader && cpuLeader.cpu >= 25 ? "ralentissement durable" : rosetta.active ? "Rosetta actif" : sleepers.length ? "veille lourde" : "normal");

  const advice = cpuLeader && cpuLeader.cpu >= 25
    ? `${cpuLeader.name} consomme beaucoup de CPU en moyenne sur la fenêtre de 5 minutes et peut ralentir le Mac.`
    : rosettaSuspects.length
      ? `${rosettaSuspects[0].name} est un candidat Rosetta Intel-only. Une version Apple Silicon native réduirait la traduction.`
    : sleepers.length
      ? `${sleepers[0].name} reste en veille avec ${sleepers[0].memory} de RAM en moyenne : candidat à fermer si tu ne l'utilises plus.`
      : "Aucune app ne ressort comme fortement bloquante sur la moyenne glissante.";
  setText("process-advice", advice);

  renderActiveProcessList();
}

function updateThermal(data) {
  const topHeat = data.processes?.topHeat?.[0];
  const heatIndex = Math.max(
    0,
    Math.min(100, Math.round(Math.max(data.cpu.usage * 0.72, topHeat?.heat || 0)))
  );
  const hasTemperature = data.thermal?.temperatureC != null;

  setText("thermal-mode", hasTemperature ? "Capteur température" : "Indice thermique");
  setText("system-temp", hasTemperature ? `${Math.round(data.thermal.temperatureC)} °C` : `${heatIndex}/100`);
  setText(
    "system-temp-note",
    hasTemperature
      ? data.thermal?.note || "Température système exposée par macOS."
      : "°C verrouillés par macOS : proxy basé sur CPU, pression thermique et top processus."
  );
  setText("thermal-summary", data.thermal?.state || data.cpu.thermalState);
  setText("thermal-source", hasTemperature ? data.thermal?.source || "macOS" : "proxy local");
  setText("thermal-top", topHeat ? `${topHeat.name} · ${Math.round(topHeat.heat)}` : "--");
  setWidth("thermal-index-bar", hasTemperature ? Math.max(0, Math.min(100, (data.thermal.temperatureC - 30) * 1.6)) : heatIndex);
}

function updateLlm(llm = {}) {
  const list = $("llm-list");
  const tools = llm.tools || [];
  const active = tools.filter((tool) => tool.active).length;
  setText("llm-active-count", `${active}/${tools.length || 3}`);
  setText("llm-note", llm.note || "Tokens live non exposés par les outils locaux.");
  if (!list) return;
  list.replaceChildren();

  tools.forEach((tool) => {
    const row = document.createElement("div");
    row.className = `llm-row ${tool.active ? "is-active" : "is-idle"}`;
    const interfaceLabel = labelInterface(tool.interface);
    const session = usageLine(tool, "Session") || usageLine(tool, "Pro") || usageLine(tool, "Spark");
    const weekly = usageLine(tool, "Weekly") || usageLine(tool, "Spark Weekly") || usageLine(tool, "Flash");
    const today = usageLine(tool, "Today");
    const tokenText = today?.value || shortTokenStatus(tool.tokenStatus);
    row.innerHTML = `
      <div class="llm-name"><span>${escapeHtml(tool.name)}</span><small>${escapeHtml(tool.plan || (tool.active ? "actif" : "idle"))}</small></div>
      <div><strong>${escapeHtml(interfaceLabel)}</strong><small>interface</small></div>
      <div><strong>${escapeHtml(session ? usageValue(session) : "--")}</strong><small>${escapeHtml(session?.label || "session")}</small></div>
      <div><strong>${escapeHtml(weekly ? usageValue(weekly) : "--")}</strong><small>${escapeHtml(weekly?.label || "quota")}</small></div>
      <div class="llm-token"><strong>${escapeHtml(tokenText)}</strong><small>${escapeHtml(tool.usageSource === "OpenUsage" ? "OpenUsage" : "tokens")}</small></div>
      <div class="llm-process"><strong>${tool.cpu.toFixed(1)}% · ${escapeHtml(tool.memory)}</strong><small>process</small></div>
    `;
    list.append(row);
  });
}

function usageLine(tool, label) {
  return (tool.usageLines || []).find((line) => line.label === label);
}

function usageValue(line) {
  if (!line) return "--";
  if (line.value) return line.value;
  if (Number.isFinite(line.percent)) return `${Math.round(line.percent)}%`;
  if (Number.isFinite(line.used) && Number.isFinite(line.limit)) return `${line.used}/${line.limit}`;
  return "--";
}

function labelInterface(value) {
  if (!value || value === "non détecté") return "non détecté";
  return value
    .replaceAll("terminal/shell", "Terminal")
    .replaceAll("app desktop", "App desktop")
    .replaceAll("processus local", "Processus local");
}

function shortTokenStatus(value) {
  if (!value) return "non exposé";
  return value.includes("source locale") ? "logs locaux" : "non exposé live";
}

function renderActiveProcessList() {
  const topCpu = latestProcesses.topCpu || [];
  const topMemory = latestProcesses.topMemory || [];
  const topHeat = latestProcesses.topHeat || [];
  const rosettaSuspects = latestProcesses.rosetta?.suspects || [];
  const map = {
    cpu: [topCpu, "cpu"],
    memory: [topMemory, "ram"],
    heat: [topHeat, "heat"],
    rosetta: [rosettaSuspects, "cpu"],
  };
  const [items, metric] = map[processTab] || map.cpu;
  document.querySelectorAll("[data-process-tab]").forEach((button) => {
    const active = button.dataset.processTab === processTab;
    button.classList.toggle("is-active", active);
    button.setAttribute("aria-selected", active ? "true" : "false");
  });
  renderProcessList(items, metric);
}

function update(data) {
  const gpuProxy = Math.min(100, Math.round(data.cpu.usage * 0.58 + data.memory.pressure * 0.22));
  push(history.cpu, data.cpu.usage);
  push(history.gpu, gpuProxy);
  push(history.down, data.network.downloadBps);
  push(history.up, data.network.uploadBps);

  setText("subtitle", `${data.static.model} · ${data.static.hostname}`);
  setText("cpu-name", data.static.cpuName);
  setText("cpu-usage", Math.round(data.cpu.usage));
  setText("cpu-load", data.cpu.load1.toFixed(2));
  setText("cpu-processes", data.cpu.processes);
  setText("cpu-thermal", data.cpu.thermalState);
  setGauge("cpu-gauge", data.cpu.usage);

  setText("gpu-usage", gpuProxy);
  setGauge("gpu-gauge", gpuProxy);
  setText("gpu-note", data.cpu.thermalState === "Not exposed" ? "privé" : "exposé");

  setText("ram-percent", `${Math.round(data.memory.pressure)}%`);
  setText("ram-used", `${data.human.memoryUsed} / ${data.human.memoryTotal}`);
  setText("ram-active", bytes(data.memory.active));
  setText("ram-wired", bytes(data.memory.wired));
  setText("ram-compressed", bytes(data.memory.compressed));
  setWidth("ram-bar", data.memory.pressure);

  setText("disk-percent", `${Math.round(data.disk.percent)}%`);
  setText("disk-used", `${data.human.diskUsed} / ${data.human.diskTotal}`);
  setText("disk-free", data.human.diskAvailable);
  setText("disk-fs", data.static.filesystem);
  setText("disk-smart", data.static.smartStatus);
  setWidth("disk-bar", data.disk.percent);

  setText("wifi-name", data.network.wifi);
  setText("download", data.human.download);
  setText("upload", data.human.upload);
  setText("total-rx", data.human.totalRx);
  setText("total-tx", data.human.totalTx);
  setText("hostname", data.static.hostname);

  setText("battery-percent", `${data.battery.percent}%`);
  setText("battery-status", data.battery.status);
  setText("power-source", data.battery.source);
  setText("battery-time", data.battery.remaining);
  setText("battery-cycle", data.battery.cycleCount);
  setText("battery-health", data.battery.condition);
  setText("battery-health-2", data.battery.condition);
  setWidth("battery-bar", data.battery.percent);

  updateThermal(data);
  updateLlm(data.llm);
  setText("model", data.static.model);
  setText("cpu-cores", `${data.static.cpuCores} coeurs logiques`);
  setText("uptime", data.uptime.display);
  setText("updated", formatTime(data.timestamp));

  setText("overall-health", translateHealth(data.health.status));
  setText("header-health", data.health.status === "Good" ? "Système sain" : "À surveiller");
  setText("storage-health", data.static.smartStatus);
  setText("pressure-health", `${Math.round(data.memory.pressure)}%`);
  setText("thermal-health", data.cpu.thermalState);
  setText("footer-health", data.health.status === "Good" ? "Système sain" : "À surveiller");
  setText("footer-note", data.health.warnings.length ? data.health.warnings.map(translateWarning).join(", ") : "Tout est normal");
  setText("footer-uptime", data.uptime.display);
  setText("footer-read", data.human.totalRx);
  setText("footer-written", data.human.totalTx);
  updateProcesses(data.processes);

  drawChart("cpu-chart", history.cpu, "#1685e8", 100);
  drawChart("gpu-chart", history.gpu, "#33aa45", 100);
  drawNetChart("download-chart", history.down, "#1685e8");
  drawNetChart("upload-chart", history.up, "#33aa45");
}

function bytes(value) {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = Math.max(0, value);
  let index = 0;
  while (size >= 1024 && index < units.length - 1) {
    size /= 1024;
    index += 1;
  }
  return `${size.toFixed(index ? 1 : 0)} ${units[index]}`;
}

async function tick() {
  if (paused) return;
  try {
    const response = await fetch("/api/metrics", { cache: "no-store" });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    update(await response.json());
  } catch (error) {
    setText("footer-health", "Offline");
    setText("footer-note", "Backend metrics unavailable");
  }
}

$("refresh").addEventListener("click", tick);
$("pause").addEventListener("click", () => {
  paused = !paused;
  $("pause").classList.toggle("is-paused", paused);
});

document.querySelectorAll("[data-process-tab]").forEach((button) => {
  button.addEventListener("click", () => {
    processTab = button.dataset.processTab;
    renderActiveProcessList();
  });
});

window.addEventListener("resize", () => {
  drawChart("cpu-chart", history.cpu, "#1685e8", 100);
  drawChart("gpu-chart", history.gpu, "#33aa45", 100);
  drawNetChart("download-chart", history.down, "#1685e8");
  drawNetChart("upload-chart", history.up, "#33aa45");
});

tick();
timer = window.setInterval(tick, 2000);
