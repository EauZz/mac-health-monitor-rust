use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::env;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

const DEFAULT_PORT: u16 = 8767;
const PROCESS_WINDOW_SECS: u64 = 300;

#[derive(Clone, Default)]
struct StaticInfo {
    hostname: String,
    model: String,
    cpu_name: String,
    cpu_cores: usize,
    memory_total: u64,
    disk_name: String,
    filesystem: String,
    smart_status: String,
}

#[derive(Clone, Default)]
struct PowerInfo {
    cycle_count: String,
    condition: String,
    max_capacity: String,
}

struct AppState {
    last_network_at: Option<Instant>,
    last_rx: u64,
    last_tx: u64,
    static_at: Option<Instant>,
    static_info: StaticInfo,
    power_at: Option<Instant>,
    power_info: PowerInfo,
    process_history: HashMap<String, VecDeque<ProcessSample>>,
    arch_cache: HashMap<String, String>,
}

impl AppState {
    fn new() -> Self {
        Self {
            last_network_at: None,
            last_rx: 0,
            last_tx: 0,
            static_at: None,
            static_info: StaticInfo::default(),
            power_at: None,
            power_info: PowerInfo::default(),
            process_history: HashMap::new(),
            arch_cache: HashMap::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let requested_port = env::var("MAC_HEALTH_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);
    let listener = TcpListener::bind(("127.0.0.1", requested_port))
        .or_else(|_| TcpListener::bind(("127.0.0.1", 0)))?;
    let port = listener.local_addr()?.port();
    let state = Arc::new(Mutex::new(AppState::new()));
    let url = format!("http://127.0.0.1:{port}");

    std::thread::spawn(move || run_server(listener, state));

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Mac Health Monitor Rust")
        .with_inner_size(LogicalSize::new(1180.0, 920.0))
        .with_min_inner_size(LogicalSize::new(820.0, 640.0))
        .build(&event_loop)?;
    let _webview = WebViewBuilder::new().with_url(&url).build(&window)?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}

fn run_server(listener: TcpListener, state: Arc<Mutex<AppState>>) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = Arc::clone(&state);
                std::thread::spawn(move || {
                    if let Err(error) = handle_connection(stream, state) {
                        eprintln!("request failed: {error}");
                    }
                });
            }
            Err(error) => eprintln!("connection failed: {error}"),
        }
    }
}

fn handle_connection(mut stream: TcpStream, state: Arc<Mutex<AppState>>) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;
    let path = first_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .split('?')
        .next()
        .unwrap_or("/");

    if path == "/api/metrics" {
        let body = {
            let mut state = state.lock().expect("state poisoned");
            metrics_json(&mut state)
        };
        return write_response(
            &mut stream,
            "200 OK",
            "application/json; charset=utf-8",
            body.as_bytes(),
        );
    }

    let public = public_dir();
    let mut file_path = if path == "/" {
        public.join("index.html")
    } else {
        public.join(path.trim_start_matches('/'))
    };

    if !file_path.starts_with(&public) {
        return write_response(
            &mut stream,
            "403 Forbidden",
            "text/plain; charset=utf-8",
            b"Forbidden",
        );
    }

    if file_path.is_dir() {
        file_path = file_path.join("index.html");
    }

    match fs::read(&file_path) {
        Ok(body) => write_response(&mut stream, "200 OK", mime_for(&file_path), &body),
        Err(_) => write_response(
            &mut stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            b"Not found",
        ),
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)
}

fn public_dir() -> PathBuf {
    if let Ok(exe) = env::current_exe() {
        if let Some(macos_dir) = exe.parent() {
            let bundled = macos_dir.join("../Resources/app/public");
            if bundled.exists() {
                return normalize(bundled);
            }
            let local = macos_dir.join("public");
            if local.exists() {
                return normalize(local);
            }
        }
    }
    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("public")
}

fn normalize(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).unwrap_or(path)
}

fn mime_for(path: &PathBuf) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        _ => "application/octet-stream",
    }
}

fn metrics_json(state: &mut AppState) -> String {
    let static_info = static_info(state);
    let cpu = cpu_metrics();
    let memory = memory_metrics(static_info.memory_total);
    let disk = disk_metrics();
    let battery = battery_metrics(state);
    let network = network_metrics(state);
    let uptime = uptime_metrics();
    let thermal = thermal_metrics(&cpu.thermal_state);
    let process_report = process_metrics(state);
    let timestamp = unix_time();
    let mut warnings = Vec::new();

    if cpu.usage > 85.0 {
        warnings.push("CPU high");
    }
    if memory.pressure > 85.0 {
        warnings.push("Memory pressure high");
    }
    if disk.percent > 85.0 {
        warnings.push("Disk nearly full");
    }
    if battery.condition != "Unknown" && battery.condition != "Normal" {
        warnings.push("Battery condition");
    }
    if process_report.max_cpu >= 25.0 {
        warnings.push("Process CPU high");
    }

    let health_status = if warnings.is_empty() { "Good" } else { "Check" };
    let warnings_json = warnings
        .iter()
        .map(|warning| json_string(warning))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"timestamp\":{timestamp},\"static\":{{\"hostname\":{},\"model\":{},\"cpuName\":{},\"cpuCores\":{},\"memoryTotal\":{},\"diskName\":{},\"filesystem\":{},\"smartStatus\":{}}},\"cpu\":{{\"usage\":{},\"load1\":{},\"load5\":{},\"load15\":{},\"processes\":{},\"thermalState\":{}}},\"memory\":{{\"total\":{},\"used\":{},\"free\":{},\"standby\":{},\"active\":{},\"wired\":{},\"compressed\":{},\"pressure\":{}}},\"disk\":{{\"used\":{},\"total\":{},\"available\":{},\"percent\":{}}},\"battery\":{{\"percent\":{},\"status\":{},\"source\":{},\"remaining\":{},\"cycleCount\":{},\"condition\":{},\"maxCapacity\":{}}},\"network\":{{\"downloadBps\":{},\"uploadBps\":{},\"totalRx\":{},\"totalTx\":{},\"wifi\":{}}},\"uptime\":{{\"seconds\":{},\"display\":{}}},\"thermal\":{},\"health\":{{\"status\":{},\"warnings\":[{}]}},\"processes\":{},\"llm\":{},\"human\":{{\"memoryUsed\":{},\"memoryTotal\":{},\"diskUsed\":{},\"diskTotal\":{},\"diskAvailable\":{},\"download\":{},\"upload\":{},\"totalRx\":{},\"totalTx\":{}}}}}",
        json_string(&static_info.hostname),
        json_string(&static_info.model),
        json_string(&static_info.cpu_name),
        static_info.cpu_cores,
        static_info.memory_total,
        json_string(&static_info.disk_name),
        json_string(&static_info.filesystem),
        json_string(&static_info.smart_status),
        fmt_num(cpu.usage),
        fmt_num(cpu.load1),
        fmt_num(cpu.load5),
        fmt_num(cpu.load15),
        cpu.processes,
        json_string(&cpu.thermal_state),
        memory.total,
        memory.used,
        memory.free,
        memory.standby,
        memory.active,
        memory.wired,
        memory.compressed,
        fmt_num(memory.pressure),
        disk.used,
        disk.total,
        disk.available,
        fmt_num(disk.percent),
        battery.percent,
        json_string(&battery.status),
        json_string(&battery.source),
        json_string(&battery.remaining),
        json_string(&battery.cycle_count),
        json_string(&battery.condition),
        json_string(&battery.max_capacity),
        fmt_num(network.download_bps),
        fmt_num(network.upload_bps),
        network.total_rx,
        network.total_tx,
        json_string(&network.wifi),
        uptime.seconds,
        json_string(&uptime.display),
        thermal.to_json(),
        json_string(health_status),
        warnings_json,
        process_report.to_json(),
        process_report.llm.to_json(),
        json_string(&bytes_human(memory.used as f64)),
        json_string(&bytes_human(memory.total as f64)),
        json_string(&bytes_human(disk.used as f64)),
        json_string(&bytes_human(disk.total as f64)),
        json_string(&bytes_human(disk.available as f64)),
        json_string(&format!("{}/s", bytes_human(network.download_bps))),
        json_string(&format!("{}/s", bytes_human(network.upload_bps))),
        json_string(&bytes_human(network.total_rx as f64)),
        json_string(&bytes_human(network.total_tx as f64)),
    )
}

#[derive(Default)]
struct CpuMetrics {
    usage: f64,
    load1: f64,
    load5: f64,
    load15: f64,
    processes: usize,
    thermal_state: String,
}

#[derive(Default)]
struct MemoryMetrics {
    total: u64,
    used: u64,
    free: u64,
    standby: u64,
    active: u64,
    wired: u64,
    compressed: u64,
    pressure: f64,
}

#[derive(Default)]
struct DiskMetrics {
    used: u64,
    total: u64,
    available: u64,
    percent: f64,
}

#[derive(Default)]
struct BatteryMetrics {
    percent: u64,
    status: String,
    source: String,
    remaining: String,
    cycle_count: String,
    condition: String,
    max_capacity: String,
}

#[derive(Default)]
struct NetworkMetrics {
    download_bps: f64,
    upload_bps: f64,
    total_rx: u64,
    total_tx: u64,
    wifi: String,
}

#[derive(Default)]
struct UptimeMetrics {
    seconds: u64,
    display: String,
}

#[derive(Default)]
struct ThermalMetrics {
    temperature_c: Option<f64>,
    state: String,
    source: String,
    privileged_required: bool,
    note: String,
}

impl ThermalMetrics {
    fn to_json(&self) -> String {
        let temperature = self
            .temperature_c
            .map(fmt_num)
            .unwrap_or_else(|| "null".to_string());
        format!(
            "{{\"temperatureC\":{},\"state\":{},\"source\":{},\"privilegedRequired\":{},\"note\":{}}}",
            temperature,
            json_string(&self.state),
            json_string(&self.source),
            if self.privileged_required {
                "true"
            } else {
                "false"
            },
            json_string(&self.note),
        )
    }
}

#[derive(Clone, Default)]
struct ProcessInfo {
    pid: u64,
    name: String,
    cpu: f64,
    memory_bytes: u64,
    state: String,
    heat: f64,
    hint: String,
    badge: String,
    cause: String,
    sleeping: bool,
    process_count: usize,
    samples: usize,
    window_seconds: u64,
    architecture: String,
    rosetta_candidate: bool,
    interface: String,
}

#[derive(Clone, Default)]
struct ProcessSample {
    at: u64,
    pid: u64,
    cpu: f64,
    memory_bytes: u64,
    state: String,
    heat: f64,
    sleeping: bool,
    cause: String,
    process_count: usize,
    architecture: String,
    rosetta_candidate: bool,
    interface: String,
}

#[derive(Default)]
struct ProcessReport {
    top_cpu: Vec<ProcessInfo>,
    top_memory: Vec<ProcessInfo>,
    top_heat: Vec<ProcessInfo>,
    sleepers: Vec<ProcessInfo>,
    rosetta: RosettaReport,
    llm: LlmReport,
    max_cpu: f64,
}

#[derive(Default)]
struct RosettaReport {
    active: bool,
    oahd_cpu: f64,
    suspects: Vec<ProcessInfo>,
    note: String,
}

#[derive(Default)]
struct LlmReport {
    tools: Vec<LlmTool>,
    active_count: usize,
    note: String,
}

#[derive(Default)]
struct LlmTool {
    name: String,
    active: bool,
    interface: String,
    cpu: f64,
    memory_bytes: u64,
    process_count: usize,
    token_status: String,
    detail: String,
    usage_source: String,
    plan: String,
    fetched_at: String,
    usage_lines: Vec<LlmUsageLine>,
}

#[derive(Clone, Default)]
struct LlmUsageLine {
    line_type: String,
    label: String,
    value: String,
    used: Option<f64>,
    limit: Option<f64>,
    percent: Option<f64>,
    format_kind: String,
    resets_at: String,
}

impl ProcessReport {
    fn to_json(&self) -> String {
        format!(
            "{{\"mode\":\"rolling\",\"windowSeconds\":{},\"topCpu\":{},\"topMemory\":{},\"topHeat\":{},\"sleepers\":{},\"rosetta\":{}}}",
            PROCESS_WINDOW_SECS,
            process_list_json(&self.top_cpu),
            process_list_json(&self.top_memory),
            process_list_json(&self.top_heat),
            process_list_json(&self.sleepers),
            self.rosetta.to_json(),
        )
    }
}

impl RosettaReport {
    fn to_json(&self) -> String {
        format!(
            "{{\"active\":{},\"oahdCpu\":{},\"suspects\":{},\"note\":{}}}",
            if self.active { "true" } else { "false" },
            fmt_num(self.oahd_cpu),
            process_list_json(&self.suspects),
            json_string(&self.note),
        )
    }
}

impl LlmReport {
    fn to_json(&self) -> String {
        let tools = self
            .tools
            .iter()
            .map(llm_tool_json)
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"activeCount\":{},\"tools\":[{}],\"note\":{}}}",
            self.active_count,
            tools,
            json_string(&self.note),
        )
    }
}

fn static_info(state: &mut AppState) -> StaticInfo {
    if let Some(last) = state.static_at {
        if last.elapsed() < Duration::from_secs(120) {
            return state.static_info.clone();
        }
    }

    let cpu_name = read_sysctl("machdep.cpu.brand_string")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Apple Silicon".to_string());
    let model = read_sysctl("hw.model")
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Mac".to_string());
    let memory_total = read_sysctl("hw.memsize")
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(0);
    let hostname = run(&["scutil", "--get", "LocalHostName"])
        .lines()
        .next()
        .map(|value| format!("{value}.local"))
        .unwrap_or_else(|| "Mac.local".to_string());

    let disk_info = run(&["diskutil", "info", "/"]);
    let mut disk_name = "Internal SSD".to_string();
    let mut filesystem = "APFS".to_string();
    let mut smart_status = "Unavailable".to_string();

    for line in disk_info.lines() {
        let clean = line.trim();
        if let Some(value) = clean.strip_prefix("Device / Media Name:") {
            disk_name = value.trim().to_string();
        } else if let Some(value) = clean.strip_prefix("File System Personality:") {
            filesystem = value.trim().to_string();
        } else if let Some(value) = clean.strip_prefix("SMART Status:") {
            smart_status = value.trim().to_string();
        }
    }

    let info = StaticInfo {
        hostname,
        model,
        cpu_name,
        cpu_cores: std::thread::available_parallelism()
            .map(|value| value.get())
            .unwrap_or(1),
        memory_total,
        disk_name,
        filesystem,
        smart_status,
    };
    state.static_at = Some(Instant::now());
    state.static_info = info.clone();
    info
}

fn cpu_metrics() -> CpuMetrics {
    let cpu_total = run(&["ps", "-A", "-o", "%cpu="])
        .lines()
        .map(to_f64)
        .sum::<f64>();
    let cores = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1) as f64;
    let usage = (cpu_total / cores).clamp(0.0, 100.0);
    let processes = run(&["ps", "-A", "-o", "pid="]).lines().count();
    let load = run(&["sysctl", "-n", "vm.loadavg"]);
    let loads = load
        .split(|char: char| char.is_whitespace() || char == '{' || char == '}')
        .filter_map(|value| value.parse::<f64>().ok())
        .collect::<Vec<_>>();
    let therm = run(&["pmset", "-g", "therm"]);
    let thermal_state = if let Some(limit) = value_after(&therm, "CPU_Speed_Limit") {
        if limit < 100.0 {
            format!("Limited {:.0}%", limit)
        } else {
            "Nominal".to_string()
        }
    } else if therm.contains("Error:") {
        "Not exposed".to_string()
    } else {
        "Nominal".to_string()
    };

    CpuMetrics {
        usage: round1(usage),
        load1: round2(*loads.first().unwrap_or(&0.0)),
        load5: round2(*loads.get(1).unwrap_or(&0.0)),
        load15: round2(*loads.get(2).unwrap_or(&0.0)),
        processes,
        thermal_state,
    }
}

fn memory_metrics(total_hint: u64) -> MemoryMetrics {
    let out = run(&["vm_stat"]);
    let page_size = between(&out, "page size of ", " bytes")
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(16_384);
    let mut pages = HashMap::new();
    for line in out.lines() {
        if let Some((key, raw)) = line.split_once(':') {
            let value = raw
                .chars()
                .filter(|char| char.is_ascii_digit())
                .collect::<String>()
                .parse::<u64>()
                .unwrap_or(0);
            pages.insert(key.trim().to_string(), value);
        }
    }

    let free_pages = pages.get("Pages free").copied().unwrap_or(0)
        + pages.get("Pages speculative").copied().unwrap_or(0);
    let active = pages.get("Pages active").copied().unwrap_or(0) * page_size;
    let inactive = pages.get("Pages inactive").copied().unwrap_or(0) * page_size;
    let wired = pages.get("Pages wired down").copied().unwrap_or(0) * page_size;
    let compressed = pages
        .get("Pages occupied by compressor")
        .copied()
        .unwrap_or(0)
        * page_size;
    let free_raw = free_pages * page_size;
    let total = if total_hint > 0 {
        total_hint
    } else {
        active + inactive + wired + compressed + free_raw
    };
    let used = total.min(active + wired + compressed);
    let free = total.saturating_sub(used);
    let pressure = if total > 0 {
        ((used as f64 / total as f64) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    MemoryMetrics {
        total,
        used,
        free,
        standby: inactive + free_raw,
        active,
        wired,
        compressed,
        pressure: round1(pressure),
    }
}

fn disk_metrics() -> DiskMetrics {
    let target = if PathBuf::from("/System/Volumes/Data").exists() {
        "/System/Volumes/Data"
    } else {
        "/"
    };
    let out = run(&["df", "-k", target]);
    let Some(line) = out.lines().nth(1) else {
        return DiskMetrics::default();
    };
    let parts = line.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 4 {
        return DiskMetrics::default();
    }
    let total = parts[1].parse::<u64>().unwrap_or(0) * 1024;
    let used = parts[2].parse::<u64>().unwrap_or(0) * 1024;
    let available = parts[3].parse::<u64>().unwrap_or(0) * 1024;
    let percent = if total > 0 {
        (used as f64 / total as f64) * 100.0
    } else {
        0.0
    };
    DiskMetrics {
        used,
        total,
        available,
        percent: round1(percent),
    }
}

fn battery_metrics(state: &mut AppState) -> BatteryMetrics {
    let batt = run(&["pmset", "-g", "batt"]);
    let mut percent = 0;
    let mut status = "Unknown".to_string();
    let mut source = "Unknown".to_string();
    let mut remaining = "Unknown".to_string();

    if let Some(first) = batt.lines().next() {
        if let Some(value) = between(first, "'", "'") {
            source = value;
        }
    }
    for line in batt.lines() {
        if let Some((before, after)) = line.split_once("%;") {
            percent = last_number(before);
            let parts = after.split(';').map(str::trim).collect::<Vec<_>>();
            if let Some(value) = parts.first() {
                status = capitalize(value);
            }
            if let Some(value) = parts.get(1) {
                remaining = value.replace("present: true", "").trim().to_string();
            }
        }
    }

    let power = power_info(state);
    BatteryMetrics {
        percent,
        status,
        source,
        remaining,
        cycle_count: power.cycle_count,
        condition: power.condition,
        max_capacity: power.max_capacity,
    }
}

fn power_info(state: &mut AppState) -> PowerInfo {
    if let Some(last) = state.power_at {
        if last.elapsed() < Duration::from_secs(90) {
            return state.power_info.clone();
        }
    }

    let out = run(&["system_profiler", "SPPowerDataType", "-detailLevel", "mini"]);
    let mut info = PowerInfo {
        cycle_count: "Unknown".to_string(),
        condition: "Unknown".to_string(),
        max_capacity: "Unknown".to_string(),
    };
    for line in out.lines() {
        let clean = line.trim();
        if let Some(value) = clean.strip_prefix("Cycle Count:") {
            info.cycle_count = value.trim().to_string();
        } else if let Some(value) = clean.strip_prefix("Condition:") {
            info.condition = value.trim().to_string();
        } else if let Some(value) = clean.strip_prefix("Maximum Capacity:") {
            info.max_capacity = value.trim().to_string();
        }
    }

    state.power_at = Some(Instant::now());
    state.power_info = info.clone();
    info
}

fn network_metrics(state: &mut AppState) -> NetworkMetrics {
    let now = Instant::now();
    let (rx, tx) = network_totals();
    let (download_bps, upload_bps) = match state.last_network_at {
        Some(last) => {
            let elapsed = now.duration_since(last).as_secs_f64().max(0.1);
            (
                rx.saturating_sub(state.last_rx) as f64 / elapsed,
                tx.saturating_sub(state.last_tx) as f64 / elapsed,
            )
        }
        None => (0.0, 0.0),
    };
    state.last_network_at = Some(now);
    state.last_rx = rx;
    state.last_tx = tx;

    let wifi_raw = run(&["networksetup", "-getairportnetwork", "en0"]);
    let wifi = wifi_raw
        .split_once(':')
        .map(|(_, value)| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| wifi_raw.trim().to_string());

    NetworkMetrics {
        download_bps,
        upload_bps,
        total_rx: rx,
        total_tx: tx,
        wifi: if wifi.is_empty() {
            "Unknown".to_string()
        } else {
            wifi
        },
    }
}

fn network_totals() -> (u64, u64) {
    let out = run(&["netstat", "-ibn"]);
    let mut rx = 0;
    let mut tx = 0;
    let mut seen = HashSet::new();
    for line in out.lines().skip(1) {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() < 10 {
            continue;
        }
        let name = parts[0];
        if seen.contains(name)
            || name.starts_with("lo")
            || name.starts_with("utun")
            || name.starts_with("awdl")
            || name.starts_with("llw")
        {
            continue;
        }
        seen.insert(name.to_string());
        rx += parts[6].parse::<u64>().unwrap_or(0);
        tx += parts[9].parse::<u64>().unwrap_or(0);
    }
    (rx, tx)
}

fn uptime_metrics() -> UptimeMetrics {
    let out = run(&["sysctl", "-n", "kern.boottime"]);
    let seconds = between(&out, "sec = ", ",")
        .and_then(|value| value.parse::<u64>().ok())
        .map(|boot| unix_time().saturating_sub(boot))
        .unwrap_or(0);
    UptimeMetrics {
        seconds,
        display: seconds_human(seconds),
    }
}

fn thermal_metrics(cpu_thermal_state: &str) -> ThermalMetrics {
    let pmset = run(&["pmset", "-g", "therm"]);
    if let Some(value) = first_temperature_c(&pmset) {
        return ThermalMetrics {
            temperature_c: Some(value),
            state: cpu_thermal_state.to_string(),
            source: "pmset".to_string(),
            privileged_required: false,
            note: "Température exposée par macOS.".to_string(),
        };
    }

    if env::var("MAC_HEALTH_PRIVILEGED_THERMAL").ok().as_deref() == Some("1") {
        let power = run(&[
            "powermetrics",
            "--samplers",
            "thermal",
            "-n",
            "1",
            "-i",
            "1000",
        ]);
        if let Some(value) = first_temperature_c(&power) {
            return ThermalMetrics {
                temperature_c: Some(value),
                state: cpu_thermal_state.to_string(),
                source: "powermetrics".to_string(),
                privileged_required: false,
                note: "Température lue via powermetrics.".to_string(),
            };
        }
    }

    ThermalMetrics {
        temperature_c: None,
        state: cpu_thermal_state.to_string(),
        source: "macOS restricted".to_string(),
        privileged_required: true,
        note: "Apple Silicon ne publie pas la température en °C aux apps normales.".to_string(),
    }
}

fn process_metrics(state: &mut AppState) -> ProcessReport {
    let now = unix_time();
    let current = current_process_groups(&mut state.arch_cache);

    for item in current {
        let samples = state.process_history.entry(item.name.clone()).or_default();
        samples.push_back(ProcessSample {
            at: now,
            pid: item.pid,
            cpu: item.cpu,
            memory_bytes: item.memory_bytes,
            state: item.state,
            heat: item.heat,
            sleeping: item.sleeping,
            cause: item.cause,
            process_count: item.process_count,
            architecture: item.architecture,
            rosetta_candidate: item.rosetta_candidate,
            interface: item.interface,
        });
    }

    for samples in state.process_history.values_mut() {
        while samples
            .front()
            .is_some_and(|sample| sample.at + PROCESS_WINDOW_SECS < now)
        {
            samples.pop_front();
        }
    }
    state
        .process_history
        .retain(|_, samples| !samples.is_empty());

    let mut averaged = Vec::new();
    for (name, samples) in &state.process_history {
        let count = samples.len().max(1);
        let first_at = samples.front().map(|sample| sample.at).unwrap_or(now);
        let last = samples.back().cloned().unwrap_or_default();
        let cpu = samples.iter().map(|sample| sample.cpu).sum::<f64>() / count as f64;
        let memory_bytes = samples
            .iter()
            .map(|sample| sample.memory_bytes)
            .sum::<u64>()
            / count as u64;
        let heat = samples.iter().map(|sample| sample.heat).sum::<f64>() / count as f64;
        let sleeping_ratio =
            samples.iter().filter(|sample| sample.sleeping).count() as f64 / count as f64;
        let sleeping = sleeping_ratio >= 0.75;
        let rosetta_candidate = samples.iter().any(|sample| sample.rosetta_candidate);
        let cause = samples
            .iter()
            .rev()
            .find(|sample| !sample.cause.is_empty())
            .map(|sample| sample.cause.clone())
            .unwrap_or_default();
        let interface = samples
            .iter()
            .rev()
            .find(|sample| !sample.interface.is_empty())
            .map(|sample| sample.interface.clone())
            .unwrap_or_default();
        let architecture = samples
            .iter()
            .rev()
            .find(|sample| sample.architecture != "unknown")
            .map(|sample| sample.architecture.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let (hint, badge) = process_hint(cpu, memory_bytes, sleeping);
        averaged.push(ProcessInfo {
            pid: last.pid,
            name: name.clone(),
            cpu: round2(cpu),
            memory_bytes,
            state: last.state,
            heat: round1(heat),
            hint,
            badge,
            cause,
            sleeping,
            process_count: last.process_count,
            samples: count,
            window_seconds: now.saturating_sub(first_at).min(PROCESS_WINDOW_SECS),
            architecture,
            rosetta_candidate,
            interface,
        });
    }

    let max_cpu = averaged.iter().map(|item| item.cpu).fold(0.0, f64::max);
    let mut top_cpu = averaged.clone();
    top_cpu.sort_by(|a, b| {
        b.cpu
            .partial_cmp(&a.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    top_cpu.truncate(10);

    let mut top_memory = averaged.clone();
    top_memory.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    top_memory.truncate(10);

    let mut top_heat = averaged.clone();
    top_heat.sort_by(|a, b| {
        b.heat
            .partial_cmp(&a.heat)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    top_heat.truncate(10);

    let rosetta = rosetta_report(&averaged);
    let llm = llm_report(&averaged);
    let mut sleepers = averaged
        .into_iter()
        .filter(|item| item.sleeping && item.cpu < 1.0 && item.memory_bytes > 250 * 1024 * 1024)
        .collect::<Vec<_>>();
    sleepers.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    sleepers.truncate(10);

    ProcessReport {
        top_cpu,
        top_memory,
        top_heat,
        sleepers,
        rosetta,
        llm,
        max_cpu,
    }
}

fn current_process_groups(arch_cache: &mut HashMap<String, String>) -> Vec<ProcessInfo> {
    let out = run(&["ps", "-axo", "pid=,pcpu=,rss=,stat=,comm="]);
    let mut groups: HashMap<String, ProcessInfo> = HashMap::new();
    let mut observed_commands = Vec::new();

    for line in out.lines() {
        let mut parts = line.split_whitespace();
        let Some(pid_raw) = parts.next() else {
            continue;
        };
        let Some(cpu_raw) = parts.next() else {
            continue;
        };
        let Some(rss_raw) = parts.next() else {
            continue;
        };
        let Some(state_raw) = parts.next() else {
            continue;
        };
        let command = parts.collect::<Vec<_>>().join(" ");
        observed_commands.push(command.to_ascii_lowercase());
        let pid = pid_raw.parse::<u64>().unwrap_or(0);
        let cpu = round2(to_f64(cpu_raw));
        let rss_kb = rss_raw.parse::<u64>().unwrap_or(0);
        let state = state_raw.to_string();
        let process = process_name(&command);
        let name = app_name(&process);
        if pid == 0 || name.is_empty() {
            continue;
        }

        let memory_bytes = rss_kb.saturating_mul(1024);
        let sleeping = state.starts_with('S') || state.starts_with('I');
        let heat =
            round1((cpu * 1.25 + if state.starts_with('R') { 3.0 } else { 0.0 }).clamp(0.0, 100.0));
        let architecture = executable_architecture(&command, arch_cache);
        let rosetta_candidate = architecture == "Intel";
        let interface = llm_process_interface(&process, &command);
        let cause = process_cause(&process, &command, rosetta_candidate);

        let entry = groups.entry(name.clone()).or_insert_with(|| ProcessInfo {
            pid,
            name,
            state: state.clone(),
            sleeping: true,
            process_count: 0,
            architecture: architecture.clone(),
            rosetta_candidate,
            interface: interface.clone(),
            cause: cause.clone(),
            ..ProcessInfo::default()
        });
        entry.cpu += cpu;
        entry.memory_bytes = entry.memory_bytes.saturating_add(memory_bytes);
        entry.heat += heat;
        entry.process_count += 1;
        entry.sleeping = entry.sleeping && sleeping;
        entry.cause = merge_cause(&entry.cause, &cause);
        if rosetta_candidate {
            entry.rosetta_candidate = true;
            entry.architecture = "Intel".to_string();
        } else if entry.architecture == "unknown" {
            entry.architecture = architecture;
        }
        entry.interface = merge_interface(&entry.interface, &interface);
        if state.starts_with('R') || entry.pid == 0 {
            entry.pid = pid;
            entry.state = state;
        }
    }

    let file_provider_sources = detect_file_provider_sources(&observed_commands);
    if !file_provider_sources.is_empty() {
        let sources = file_provider_sources.join(", ");
        if let Some(entry) = groups.get_mut("fileproviderd") {
            entry.cause =
                format!("File Provider: sync fichiers cloud. Source détectée: {sources}.");
        }
        if let Some(entry) = groups.get_mut("fpckservice") {
            entry.cause = format!(
                "File Provider Check: réparation/vérification du provider cloud détecté: {sources}."
            );
        }
    }

    groups
        .into_values()
        .map(|mut item| {
            item.cpu = round2(item.cpu);
            item.heat = round1(item.heat.clamp(0.0, 100.0));
            let (hint, badge) = process_hint(item.cpu, item.memory_bytes, item.sleeping);
            item.hint = hint;
            item.badge = badge;
            if item.cause.is_empty() {
                item.cause = process_cause(&item.name, "", item.rosetta_candidate);
            }
            item
        })
        .collect()
}

fn rosetta_report(items: &[ProcessInfo]) -> RosettaReport {
    let oahd_cpu = items
        .iter()
        .filter(|item| item.name == "oahd" || item.name == "oahd-helper")
        .map(|item| item.cpu)
        .sum::<f64>();
    let mut suspects = items
        .iter()
        .filter(|item| item.rosetta_candidate)
        .cloned()
        .collect::<Vec<_>>();
    suspects.sort_by(|a, b| {
        b.cpu
            .partial_cmp(&a.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    suspects.truncate(5);
    let active = oahd_cpu > 0.5 || !suspects.is_empty();
    let note = if suspects.is_empty() {
        "Aucun exécutable Intel-only détecté dans le top moyen. Les apps universelles lancées sous Rosetta ne sont pas toujours identifiables sans API privée.".to_string()
    } else {
        "Candidats Rosetta détectés via exécutables Intel-only. Si oahd monte, commence par ces apps.".to_string()
    };
    RosettaReport {
        active,
        oahd_cpu: round2(oahd_cpu),
        suspects,
        note,
    }
}

fn llm_report(items: &[ProcessInfo]) -> LlmReport {
    let usage_cache = openusage_cache();
    let tools = ["Claude", "Codex", "Gemini"]
        .iter()
        .map(|name| llm_tool(name, items, usage_cache.as_ref()))
        .collect::<Vec<_>>();
    let active_count = tools.iter().filter(|tool| tool.active).count();
    LlmReport {
        tools,
        active_count,
        note: "Tokens live non exposés de façon fiable par les apps locales. L'app affiche l'activité processus et signale seulement si une source locale existe.".to_string(),
    }
}

fn llm_tool(name: &str, items: &[ProcessInfo], usage_cache: Option<&Value>) -> LlmTool {
    let matches = items
        .iter()
        .filter(|item| item.name.eq_ignore_ascii_case(name))
        .collect::<Vec<_>>();
    let cpu = round2(matches.iter().map(|item| item.cpu).sum::<f64>());
    let memory_bytes = matches.iter().map(|item| item.memory_bytes).sum::<u64>();
    let process_count = matches.iter().map(|item| item.process_count).sum::<usize>();
    let interface = matches
        .iter()
        .filter(|item| !item.interface.is_empty())
        .map(|item| item.interface.as_str())
        .collect::<Vec<_>>()
        .join(" + ");
    let active = cpu >= 0.1 || memory_bytes > 0;
    let usage = openusage_snapshot(name, usage_cache);
    let token_status = if usage.usage_lines.is_empty() {
        llm_token_status(name)
    } else {
        "OpenUsage cache".to_string()
    };
    let detail = if active {
        format!("{} · {}", bytes_human(memory_bytes as f64), token_status)
    } else {
        format!("inactif · {token_status}")
    };

    LlmTool {
        name: name.to_string(),
        active,
        interface: if interface.is_empty() {
            "non détecté".to_string()
        } else {
            dedupe_joined(&interface)
        },
        cpu,
        memory_bytes,
        process_count,
        token_status,
        detail,
        usage_source: usage.usage_source,
        plan: usage.plan,
        fetched_at: usage.fetched_at,
        usage_lines: usage.usage_lines,
    }
}

struct OpenUsageSnapshot {
    usage_source: String,
    plan: String,
    fetched_at: String,
    usage_lines: Vec<LlmUsageLine>,
}

fn openusage_cache() -> Option<Value> {
    let home = env::var_os("HOME").map(PathBuf::from)?;
    let path = home
        .join("Library")
        .join("Application Support")
        .join("com.sunstory.openusage")
        .join("usage-api-cache.json");
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn openusage_snapshot(name: &str, cache: Option<&Value>) -> OpenUsageSnapshot {
    let provider = match name {
        "Claude" => "claude",
        "Codex" => "codex",
        "Gemini" => "gemini",
        _ => "",
    };
    let Some(snapshot) = cache
        .and_then(|value| value.get("snapshots"))
        .and_then(|snapshots| snapshots.get(provider))
    else {
        return OpenUsageSnapshot {
            usage_source: "none".to_string(),
            plan: String::new(),
            fetched_at: String::new(),
            usage_lines: Vec::new(),
        };
    };

    let usage_lines = snapshot
        .get("lines")
        .and_then(|value| value.as_array())
        .map(|lines| lines.iter().filter_map(openusage_line).collect::<Vec<_>>())
        .unwrap_or_default();

    OpenUsageSnapshot {
        usage_source: "OpenUsage".to_string(),
        plan: snapshot
            .get("plan")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
        fetched_at: snapshot
            .get("fetchedAt")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
        usage_lines,
    }
}

fn openusage_line(value: &Value) -> Option<LlmUsageLine> {
    let line_type = value.get("type")?.as_str()?.to_string();
    let label = value.get("label")?.as_str()?.to_string();
    let used = value.get("used").and_then(|value| value.as_f64());
    let limit = value.get("limit").and_then(|value| value.as_f64());
    let percent = match (used, limit) {
        (Some(used), Some(limit)) if limit > 0.0 => {
            Some(round1((used / limit * 100.0).clamp(0.0, 100.0)))
        }
        _ => None,
    };
    let value_text = value
        .get("value")
        .or_else(|| value.get("text"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .or_else(|| match (used, limit) {
            (Some(used), Some(limit)) => Some(format_usage_value(used, limit, value.get("format"))),
            _ => None,
        })
        .unwrap_or_default();

    Some(LlmUsageLine {
        line_type,
        label,
        value: value_text,
        used,
        limit,
        percent,
        format_kind: value
            .get("format")
            .and_then(|format| format.get("kind"))
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
        resets_at: value
            .get("resetsAt")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

fn format_usage_value(used: f64, limit: f64, format: Option<&Value>) -> String {
    let kind = format
        .and_then(|value| value.get("kind"))
        .and_then(|value| value.as_str())
        .unwrap_or("");
    let suffix = format
        .and_then(|value| value.get("suffix"))
        .and_then(|value| value.as_str())
        .unwrap_or("");
    match kind {
        "percent" => format!("{}%", round1(used)),
        "dollars" => format!("${:.2} / ${:.2}", used, limit),
        "count" if !suffix.is_empty() => format!("{} / {} {}", round1(used), round1(limit), suffix),
        _ => format!("{} / {}", round1(used), round1(limit)),
    }
}

fn llm_token_status(name: &str) -> String {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return "tokens non exposés".to_string();
    };
    let local_source = match name {
        "Claude" => home.join(".claude").exists(),
        "Codex" => {
            home.join(".codex").join("logs_2.sqlite").exists()
                || home.join(".codex").join("session_index.jsonl").exists()
        }
        "Gemini" => home.join(".gemini").exists(),
        _ => false,
    };
    if local_source {
        "source locale détectée, tokens live non exposés".to_string()
    } else {
        "tokens non exposés".to_string()
    }
}

fn llm_tool_json(tool: &LlmTool) -> String {
    format!(
        "{{\"name\":{},\"active\":{},\"interface\":{},\"cpu\":{},\"memoryBytes\":{},\"memory\":{},\"processCount\":{},\"tokenStatus\":{},\"detail\":{},\"usageSource\":{},\"plan\":{},\"fetchedAt\":{},\"usageLines\":{}}}",
        json_string(&tool.name),
        if tool.active { "true" } else { "false" },
        json_string(&tool.interface),
        fmt_num(tool.cpu),
        tool.memory_bytes,
        json_string(&bytes_human(tool.memory_bytes as f64)),
        tool.process_count,
        json_string(&tool.token_status),
        json_string(&tool.detail),
        json_string(&tool.usage_source),
        json_string(&tool.plan),
        json_string(&tool.fetched_at),
        llm_usage_lines_json(&tool.usage_lines),
    )
}

fn llm_usage_lines_json(lines: &[LlmUsageLine]) -> String {
    let entries = lines
        .iter()
        .map(llm_usage_line_json)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{entries}]")
}

fn llm_usage_line_json(line: &LlmUsageLine) -> String {
    format!(
        "{{\"type\":{},\"label\":{},\"value\":{},\"used\":{},\"limit\":{},\"percent\":{},\"formatKind\":{},\"resetsAt\":{}}}",
        json_string(&line.line_type),
        json_string(&line.label),
        json_string(&line.value),
        opt_num(line.used),
        opt_num(line.limit),
        opt_num(line.percent),
        json_string(&line.format_kind),
        json_string(&line.resets_at),
    )
}

fn dedupe_joined(value: &str) -> String {
    let mut parts = Vec::new();
    for part in value.split(" + ").filter(|part| !part.is_empty()) {
        if !parts.contains(&part) {
            parts.push(part);
        }
    }
    parts.join(" + ")
}

fn executable_architecture(command: &str, cache: &mut HashMap<String, String>) -> String {
    if !command.starts_with('/') {
        return "unknown".to_string();
    }
    if let Some(value) = cache.get(command) {
        return value.clone();
    }
    let archs = run(&["/usr/bin/lipo", "-archs", command]).to_ascii_lowercase();
    let value = if archs.contains("x86_64") && !archs.contains("arm64") {
        "Intel"
    } else if archs.contains("x86_64") && archs.contains("arm64") {
        "Universal"
    } else if archs.contains("arm64") {
        "Apple"
    } else {
        "unknown"
    }
    .to_string();
    cache.insert(command.to_string(), value.clone());
    value
}

fn process_name(raw: &str) -> String {
    let name = raw
        .rsplit('/')
        .next()
        .unwrap_or(raw)
        .trim()
        .trim_matches('-')
        .to_string();
    if name.chars().count() > 44 {
        format!("{}...", name.chars().take(41).collect::<String>())
    } else {
        name
    }
}

fn app_owner_from_command(command: &str) -> Option<String> {
    let marker = ".app/contents/";
    let lower = command.to_ascii_lowercase();
    let marker_index = lower.find(marker)?;
    let app_path = &command[..marker_index + 4];
    let app_name = app_path.rsplit('/').next()?.trim_end_matches(".app").trim();
    if app_name.is_empty() {
        None
    } else {
        Some(app_name.to_string())
    }
}

fn app_name(process: &str) -> String {
    let lower = process.to_ascii_lowercase();
    let mapped = if lower.contains("safari") {
        "Safari"
    } else if lower.contains("google chrome") {
        "Google Chrome"
    } else if lower.contains("brave browser") {
        "Brave Browser"
    } else if lower.contains("microsoft edge") {
        "Microsoft Edge"
    } else if lower.contains("firefox") {
        "Firefox"
    } else if lower.contains("notion") {
        "Notion"
    } else if lower.contains("claude") {
        "Claude"
    } else if lower.contains("codex") {
        "Codex"
    } else if lower.contains("gemini") || lower.contains("antigravity") {
        "Gemini"
    } else if lower.contains("visual studio code") || lower == "code helper" || lower == "code" {
        "Visual Studio Code"
    } else if lower.contains("electron") {
        "Electron App"
    } else if lower.contains("webkit.webcontent") || lower.contains("safari web content") {
        "Safari"
    } else {
        process
    };

    mapped
        .replace(" Helper (Renderer)", "")
        .replace(" Helper (GPU)", "")
        .replace(" Helper", "")
}

fn llm_process_interface(process: &str, command: &str) -> String {
    let process_lower = process.to_ascii_lowercase();
    let command_lower = command.to_ascii_lowercase();
    if !(process_lower.contains("claude")
        || process_lower.contains("codex")
        || process_lower.contains("gemini")
        || process_lower.contains("antigravity"))
    {
        return String::new();
    }
    if command_lower.contains(".app/contents/") || process_lower.contains("antigravity") {
        "app desktop".to_string()
    } else if process_lower == "claude" || process_lower == "codex" || process_lower == "gemini" {
        "terminal/shell".to_string()
    } else {
        "processus local".to_string()
    }
}

fn merge_interface(current: &str, next: &str) -> String {
    if next.is_empty() || current == next {
        return current.to_string();
    }
    if current.is_empty() {
        return next.to_string();
    }
    if current.contains(next) {
        current.to_string()
    } else {
        format!("{current} + {next}")
    }
}

fn add_detected_source(sources: &mut Vec<&'static str>, name: &'static str) {
    if !sources.contains(&name) {
        sources.push(name);
    }
}

fn detect_file_provider_sources(commands: &[String]) -> Vec<&'static str> {
    let mut sources = Vec::new();
    for command in commands {
        if command.contains("clouddocs.iclouddrivefileprovider")
            || command.contains("icloud drive")
            || command.contains("bird")
        {
            add_detected_source(&mut sources, "iCloud Drive");
        }
        if command.contains("dropbox") {
            add_detected_source(&mut sources, "Dropbox");
        }
        if command.contains("onedrive") || command.contains("one drive") {
            add_detected_source(&mut sources, "OneDrive");
        }
        if command.contains("google drive") || command.contains("drivefs") {
            add_detected_source(&mut sources, "Google Drive");
        }
        if command.contains("core sync")
            || command.contains("coresync")
            || command.contains("accfindersync")
            || command.contains("creative cloud")
        {
            add_detected_source(&mut sources, "Adobe Creative Cloud");
        }
        if command.contains("box.app") || command.contains("box drive") {
            add_detected_source(&mut sources, "Box Drive");
        }
        if command.contains("nextcloud") {
            add_detected_source(&mut sources, "Nextcloud");
        }
        if command.contains("synology drive") {
            add_detected_source(&mut sources, "Synology Drive");
        }
        if command.contains("proton drive") {
            add_detected_source(&mut sources, "Proton Drive");
        }
    }
    sources
}

fn merge_cause(current: &str, next: &str) -> String {
    if next.is_empty() || current == next {
        return current.to_string();
    }
    if current.is_empty() {
        return next.to_string();
    }
    if current.contains(next) || next.contains(current) {
        return current.to_string();
    }
    current.to_string()
}

fn process_cause(process: &str, command: &str, rosetta_candidate: bool) -> String {
    let lower = process.to_ascii_lowercase();
    let command_lower = command.to_ascii_lowercase();
    let mut cause = if lower == "windowserver" {
        "Compositeur graphique macOS: fenêtres, écrans, animations, visio ou navigateur qui redessine beaucoup."
    } else if lower == "fileproviderd" {
        "File Provider: sync fichiers cloud. Cause probable: iCloud Drive, Dropbox, OneDrive, Google Drive ou Finder."
    } else if lower.contains("fpckservice") {
        "File Provider Check: vérification/réparation d’un domaine cloud après sync ou conflit de fichiers."
    } else if lower == "cloudd" {
        "iCloud/CloudKit: sync iCloud Drive, Photos, Notes, Desktop/Documents ou données d’apps Apple."
    } else if lower == "bird" {
        "iCloud Drive: upload/download de fichiers, Desktop/Documents ou fichiers évincés du disque."
    } else if lower.contains("onedrive") {
        "OneDrive: sync fichiers cloud, File Provider ou extension Finder."
    } else if lower.contains("dropbox") {
        "Dropbox: sync fichiers cloud, File Provider ou extension Finder."
    } else if lower.contains("google drive") || lower.contains("drivefs") {
        "Google Drive: sync fichiers cloud, File Provider ou disque virtuel DriveFS."
    } else if lower.contains("clouddocs.iclouddrivefileprovider") {
        "iCloud Drive File Provider: accès/sync de fichiers iCloud via Finder ou une app."
    } else if lower.contains("accfindersync")
        || lower.contains("core sync")
        || lower.contains("coresync")
    {
        "Adobe Creative Cloud: sync fichiers, extension Finder ou Core Sync."
    } else if lower == "cmux" {
        "cmux.app: app autonome active, probablement multiplexeur/session terminal. À fermer si inutilisée."
    } else if lower.contains("webkit.gpu") {
        "WebKit GPU: Safari ou app web/WKWebView. Souvent vidéo, canvas, onglet lourd ou rendu graphique."
    } else if lower.contains("webkit.webcontent") || lower.contains("safari web content") {
        "Contenu WebKit: onglet Safari ou webview d’une app. La vraie cause est une page web active."
    } else if lower.contains("webkit.networking") {
        "Réseau WebKit: Safari ou app web télécharge, streame ou synchronise en arrière-plan."
    } else if lower == "kernel_task" {
        "Noyau macOS: drivers, I/O ou protection thermique. Peut monter pour limiter la chauffe."
    } else if lower == "mds" || lower == "mds_stores" || lower.starts_with("mdworker") {
        "Spotlight: indexation de fichiers, mails, iCloud Drive ou gros changements disque."
    } else if lower == "corespotlightd" || lower == "spotlightknowledged" {
        "Spotlight/Siri: index de recherche et connaissance locale."
    } else if lower == "nsurlsessiond" {
        "Transferts arrière-plan: téléchargements/uploads lancés par apps, Safari ou iCloud."
    } else if lower == "cloudphotod" || lower == "photolibraryd" || lower == "photoanalysisd" {
        "Photos/iCloud Photos: sync, analyse locale, souvenirs, reconnaissance ou photothèque."
    } else if lower == "trustd" {
        "Certificats TLS: validation de connexions réseau, apps web, VPN ou signatures."
    } else if lower == "syspolicyd" {
        "Gatekeeper: vérification sécurité/notarisation d’apps, extensions, fichiers téléchargés ou exécutables."
    } else if lower == "filecoordinationd" {
        "Coordination fichiers: Finder, iCloud/OneDrive/Dropbox ou apps se disputent l’accès aux mêmes fichiers."
    } else if lower == "securityd" || lower == "secd" {
        "Trousseau/iCloud Keychain: mots de passe, certificats, sessions web ou sync sécurité."
    } else if lower == "lsd" || lower == "launchservicesd" {
        "Launch Services: base des apps, associations de fichiers, extensions ou ouverture de documents."
    } else if lower == "cfprefsd" {
        "Préférences macOS: lecture/écriture de réglages par une app très active."
    } else if lower == "fseventsd" {
        "FSEvents: macOS observe beaucoup de changements fichiers, souvent sync cloud, build ou indexation."
    } else if lower == "distnoted" {
        "Notifications inter-processus: beaucoup d’apps macOS échangent des événements."
    } else if lower == "runningboardd" {
        "Gestion cycle de vie apps: macOS surveille lancement, suspension et énergie des apps."
    } else if lower == "dasd" {
        "Scheduler macOS: tâches arrière-plan différées, sync, maintenance ou notifications."
    } else if lower == "airportd" {
        "Wi-Fi: scan réseau, roaming, diagnostic ou connexion instable."
    } else if lower == "locationd" {
        "Localisation: app ou service système demande la position, météo, cartes, fuseau ou automatisation."
    } else if lower == "coreaudiod" {
        "Audio: micro, haut-parleurs, visio, capture écran, plugin audio ou app de réunion."
    } else if lower == "sharingd" || lower == "rapportd" {
        "Continuité Apple: AirDrop, Handoff, AirPlay, presse-papiers universel ou appareils proches."
    } else if lower == "duetexpertd" || lower.contains("coreduet") {
        "CoreDuet: apprentissage local des usages, suggestions Siri/Spotlight et contexte système."
    } else if lower.contains("biomesyncd") || lower.contains("biomeagent") {
        "Biome: sync/organisation des événements locaux utilisés par Siri, Spotlight et Intelligence."
    } else if lower.contains("backgroundshortcutrunner") {
        "Raccourcis macOS: une automation, action rapide ou raccourci tourne en arrière-plan."
    } else if lower == "finder" {
        "Finder: copies, aperçus Quick Look, calcul tailles, dossiers cloud ou navigation fichiers."
    } else if lower == "quicklookthumbnailing" || lower.contains("quicklook") {
        "Quick Look: génération d’aperçus et miniatures de fichiers."
    } else if lower.contains("intelligenceplatformcompute") || lower == "modelmanagerd" || lower == "mlhostd" {
        "Apple Intelligence / ML local: indexation intelligente, modèles locaux ou analyse système."
    } else if lower.contains("backgroundtaskmanagement") {
        "Login items / tâches en arrière-plan: macOS vérifie les agents lancés par des apps."
    } else if lower.contains("openandsavepanelservice") {
        "Fenêtre ouvrir/enregistrer: Finder/iCloud prépare fichiers récents, dossiers ou previews."
    } else if lower == "oahd" || lower == "oahd-helper" {
        "Rosetta: traduction x86_64. La vraie app est dans l’onglet Rosetta."
    } else if lower == "openusage" {
        "Open Usage: suivi local des quotas/tokens LLM, parfois lancé via Rosetta."
    } else if lower == "node" || lower == "npm" || lower.contains("npm exec") {
        "Node.js: serveur local, outil dev, MCP ou extension lancée depuis terminal/app."
    } else if lower.contains("safari") {
        "Safari: onglets, extensions, WebKit, vidéo ou pages web actives."
    } else if lower.contains("chrome") || lower.contains("brave") || lower.contains("edge") || lower.contains("firefox") {
        "Navigateur: onglets, extensions, vidéo, WebGL ou pages web actives."
    } else if lower.contains("claude") || lower.contains("codex") || lower.contains("gemini") {
        "LLM local: session IA active, terminal/app desktop ou extension associée."
    } else {
        ""
    }
    .to_string();

    if cause.is_empty() && rosetta_candidate {
        cause = "Rosetta: exécutable Intel traduit en Apple Silicon; chercher une version native."
            .to_string();
    }
    if cause.is_empty() && command_lower.contains(".app/contents/") {
        cause = app_owner_from_command(command)
            .map(|owner| {
                if owner.eq_ignore_ascii_case(process) {
                    format!("{owner}.app: application desktop lancée directement.")
                } else {
                    format!("Helper de {owner}: processus enfant lancé par cette app.")
                }
            })
            .unwrap_or_else(|| {
                "App desktop: processus enfant ou helper lancé par cette application.".to_string()
            });
    }
    cause
}

fn process_hint(cpu: f64, memory_bytes: u64, sleeping: bool) -> (String, String) {
    let one_gb = 1024 * 1024 * 1024;
    if cpu >= 25.0 {
        ("ralentit fortement".to_string(), "à surveiller".to_string())
    } else if cpu >= 8.0 {
        ("ralentit".to_string(), "actif".to_string())
    } else if sleeping && memory_bytes >= one_gb {
        ("en veille mais lourd".to_string(), "fermable ?".to_string())
    } else if sleeping && memory_bytes >= 350 * 1024 * 1024 {
        ("en veille".to_string(), "fermable ?".to_string())
    } else {
        ("normal".to_string(), String::new())
    }
}

fn process_list_json(items: &[ProcessInfo]) -> String {
    let entries = items.iter().map(process_json).collect::<Vec<_>>().join(",");
    format!("[{entries}]")
}

fn process_json(item: &ProcessInfo) -> String {
    format!(
        "{{\"pid\":{},\"name\":{},\"cpu\":{},\"memoryBytes\":{},\"memory\":{},\"state\":{},\"heat\":{},\"hint\":{},\"badge\":{},\"cause\":{},\"sleeping\":{},\"processCount\":{},\"samples\":{},\"windowSeconds\":{},\"architecture\":{},\"rosettaCandidate\":{}}}",
        item.pid,
        json_string(&item.name),
        fmt_num(item.cpu),
        item.memory_bytes,
        json_string(&bytes_human(item.memory_bytes as f64)),
        json_string(&item.state),
        fmt_num(item.heat),
        json_string(&item.hint),
        json_string(&item.badge),
        json_string(&item.cause),
        if item.sleeping { "true" } else { "false" },
        item.process_count,
        item.samples,
        item.window_seconds,
        json_string(&item.architecture),
        if item.rosetta_candidate {
            "true"
        } else {
            "false"
        },
    )
}

fn read_sysctl(key: &str) -> Option<String> {
    let value = run(&["sysctl", "-n", key]).trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn run(args: &[&str]) -> String {
    let Some((program, rest)) = args.split_first() else {
        return String::new();
    };
    Command::new(program)
        .args(rest)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_default()
}

fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn bytes_human(value: f64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = value.max(0.0);
    let mut unit = 0;
    while size >= 1024.0 && unit < units.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{size:.0} {}", units[unit])
    } else {
        format!("{size:.1} {}", units[unit])
    }
}

fn seconds_human(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m")
    } else if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for char in value.chars() {
        match char {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            char if char.is_control() => out.push(' '),
            char => out.push(char),
        }
    }
    out.push('"');
    out
}

fn fmt_num(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

fn opt_num(value: Option<f64>) -> String {
    value.map(fmt_num).unwrap_or_else(|| "null".to_string())
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn to_f64(value: &str) -> f64 {
    value.trim().replace(',', ".").parse::<f64>().unwrap_or(0.0)
}

fn between(value: &str, start: &str, end: &str) -> Option<String> {
    let (_, tail) = value.split_once(start)?;
    let (inside, _) = tail.split_once(end)?;
    Some(inside.trim().to_string())
}

fn first_temperature_c(value: &str) -> Option<f64> {
    for line in value.lines() {
        let lower = line.to_ascii_lowercase();
        if !(lower.contains("temperature") || lower.contains("temp")) {
            continue;
        }
        let normalized = line.replace('°', " ").replace(',', ".");
        let tokens = normalized
            .split(|char: char| char.is_whitespace() || char == ':' || char == '=')
            .collect::<Vec<_>>();
        for index in 0..tokens.len() {
            let token = tokens[index].trim_end_matches('C').trim_end_matches('c');
            let Ok(number) = token.parse::<f64>() else {
                continue;
            };
            let next = tokens.get(index + 1).copied().unwrap_or("");
            if next.eq_ignore_ascii_case("c") || line.contains("°C") || line.contains(" C") {
                return Some(round1(number));
            }
        }
    }
    None
}

fn value_after(value: &str, key: &str) -> Option<f64> {
    value.lines().find_map(|line| {
        if !line.contains(key) {
            return None;
        }
        line.split('=').nth(1).map(to_f64)
    })
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn last_number(value: &str) -> u64 {
    value
        .split(|char: char| !char.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .next_back()
        .and_then(|part| part.parse::<u64>().ok())
        .unwrap_or(0)
}
