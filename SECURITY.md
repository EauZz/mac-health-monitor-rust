# Politique de sécurité

## Versions supportées

Seule la dernière version de la branche `main` est actuellement supportée.

## Signaler une vulnérabilité

Si possible, ouvrez une alerte de sécurité privée sur GitHub ou contactez directement le mainteneur.

N’ouvrez pas d’issue publique pour une vulnérabilité qui expose des fichiers locaux, tokens, arguments de ligne de commande ou données privées.

## Limites de données locales

Mac Health Monitor Rust fonctionne d’abord en local. L’app ne doit pas transmettre de métriques système, listes de processus, données OpenUsage ou usage LLM vers des services distants.

Les changements sensibles côté sécurité doivent préserver ces limites :

- pas de télémétrie distante par défaut ;
- pas de lecture de transcripts ;
- pas de collecte de clés API ou de tokens ;
- pas de helper privilégié sans revue explicite séparée.
