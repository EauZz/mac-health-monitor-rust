# Produit

## Registre

produit

## Utilisateurs

L’utilisateur principal est le propriétaire d’un MacBook Air M2 qui veut comprendre la santé de son système sans ouvrir le Terminal ni le Moniteur d’activité. Le contexte d’usage est le diagnostic rapide pendant le travail quotidien : identifier ce qui ralentit, chauffe ou vide la batterie, et savoir si une app peut être fermée sans risque.

## Objectif produit

Mac Health Monitor est un tableau de bord macOS natif et léger pour visualiser la télémétrie locale en temps réel. Il expose CPU, mémoire, stockage, réseau, batterie, état thermique, candidats Rosetta et consommation durable des processus dans une interface calme et actionnable.

Le succès se mesure à la capacité de l’utilisateur à identifier en quelques secondes le goulot d’étranglement et les principaux responsables sur la durée, sans bruit télémétrique ni fausse précision. L’app doit être rapide, native, fiable et agréable à garder ouverte.

## Personnalité de marque

Calme, précise, premium.

Le produit doit reprendre la vitesse et le focus de Raycast, la hiérarchie visuelle de Linear et la crédibilité diagnostique d’iStat Menus, tout en utilisant une surface claire et chaude plutôt qu’un cockpit technique sombre.

## Anti-références

Ne pas imiter une fausse barre de fenêtre macOS. Ne pas créer un clone de dashboard Windows. Ne pas empiler des métriques denses dans la même zone visuelle. Ne pas ajouter d’animations lourdes, de décoration gratuite ou de panneaux trop chargés en graphiques qui rendraient le diagnostic plus difficile.

## Principes design

1. Diagnostiquer avant de décorer : chaque décision visuelle doit aider l’utilisateur à comprendre plus vite la santé, la charge ou l’action possible.
2. La moyenne vaut mieux que le bruit : le classement durable sur 5 minutes doit être visuellement plus calme et plus important que les pics instantanés.
3. Doux mais exact : utiliser des surfaces chaudes et confortables tout en gardant chiffres, libellés et alertes très nets.
4. Rendre le coupable évident : si une app ralentit, chauffe ou consomme trop de mémoire, l’interface doit la rendre facile à repérer.
5. Rester léger : éviter les dépendances inutiles, ressources trop lourdes, effets coûteux et interactions qui contredisent le rôle d’un moniteur de performance.

## Accessibilité et inclusion

Viser un contraste WCAG AA pour les textes et contrôles. Ne pas dépendre uniquement de la couleur pour les états de santé. Respecter `prefers-reduced-motion`. Garder des contrôles accessibles au clavier et une typographie lisible sur petits écrans de portable.
