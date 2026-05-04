# Design

## Thème visuel

Espace de diagnostic chaleureux. L’interface doit donner l’impression d’un panneau d’instrument calme posé sur une surface crème : claire, concentrée, premium et opérationnelle sans devenir froide ou clinique.

## Palette

- Toile de fond : dégradés crème chaud, ivoire et sable clair.
- Panneaux : blanc chaud translucide, bordures subtiles et profondeur douce.
- Texte principal : graphite presque noir.
- Texte secondaire : ardoise-taupe atténué.
- Accent principal : bleu électrique pour CPU, réseau, focus et interactions principales.
- Accent santé : vert feuille pour batterie, bons états et stabilité.
- Accent attention : ambre pour vigilance et chauffe.
- Accent critique : rouge/corail pour charge durable ou états urgents.
- Accent mémoire : violet retenu uniquement quand il aide à distinguer la RAM.

## Typographie

Utiliser les polices système Apple pour la performance et la cohérence macOS. Privilégier une hiérarchie forte par taille, graisse, espacement et longueur de ligne plutôt que charger une police externe.

- Titre de page : compact, confiant, pas surdimensionné.
- Titres de cartes : courts, contrastés, immédiatement scannables.
- Valeurs métriques : tabulaires, visibles, calmes.
- Labels et indices : texte français concis avec peu de bruit visuel.

## Mise en page

La disposition par défaut est une surface produit pleine fenêtre, pas une fausse fenêtre d’app. Utiliser un rail de contenu à largeur maximale sur grands écrans, avec un espacement généreux mais efficace.

Ordre de priorité :

1. Résumé santé et actions globales.
2. CPU, mémoire, thermique et Process Watch.
3. Batterie, stockage, réseau et détails système.
4. Pied de page de statut secondaire.

`Process Watch` doit utiliser une seule liste active et ciblée plutôt que plusieurs listes dépliées en même temps. Les principaux responsables doivent avoir un rang fort, un nom lisible, une métrique claire et un indice d’action court.

## Composants

- Cartes métriques douces avec léger effet glass, sans dépendre d’un flou lourd.
- Contrôles segmentés pour les catégories de processus.
- Lignes compactes classées pour les responsables durables.
- Mini-graphiques en support visuel, jamais comme information principale.
- Chips de santé et badges avec texte plus couleur.
- Pied de page plus discret que la zone de diagnostic principale.

## Mouvement

Le mouvement doit être minimal et fonctionnel : fondus courts, transitions de valeurs, changements d’onglets et mises à jour de graphiques. Éviter les effets cinématiques. Respecter `prefers-reduced-motion`.

## Adaptation aux écrans

Les écrans de bureau et de portable sont prioritaires. L’interface doit quand même se replier proprement sur fenêtres étroites sans débordement horizontal. `Process Watch` doit rester lisible quand les cartes passent en une seule colonne.

## Notes d’implémentation

Utiliser la pile existante HTML/CSS/JavaScript vanilla. Ne pas ajouter de framework front-end ni de dépendance d’exécution. Garder les graphiques en canvas et préserver la légèreté de l’app WKWebView native.
