# Documentation: Rust defrag.exe Simulator

## Description du projet

Ce projet est un simulateur en Rust de l'utilitaire `defrag.exe` de MS-DOS 6.22, reproduisant l'interface graphique classique et le comportement de la défragmentation de disque. Il utilise la bibliothèque `ratatui` pour l'interface utilisateur et `crossterm` pour les interactions clavier.

## Structure du projet

- `src/main.rs` : Fichier principal contenant toute la logique du simulateur
- `Cargo.toml` : Fichier de configuration des dépendances du projet
- Dépendances : `ratatui`, `crossterm`, `rand`, `clap`, `rodio`

## Fonctionnalités principales

### Interface utilisateur
- Interface fidèle à l'original MS-DOS avec en-tête, grille de disque et pied de page
- Menu déroulant comme dans le programme original (Optimize, Analyze, File, Sort, Help)
- Boîte de dialogue "About" avec ASCII art
- Curseur pour naviguer dans les menus

### Simulation de défragmentation
- Initialisation du disque avec différents types de clusters (Pending, Unused, Used, Bad, Unmovable, Reading, Writing)
- Phases de simulation : Initializing, Analyzing, Defragmenting, Finished
- Animation de lecture/écriture des clusters
- Statistiques de défragmentation en temps réel

### Contrôles utilisateur
- Navigation avec les flèches directionnelles dans les menus
- F10 ou Tab : ouvrir/fermer les menus
- F1 : Afficher la boîte "About"
- 'S' : Activer/désactiver le son
- 'Q' ou Échap : Quitter
- Entrée : Valider une sélection de menu

### Affichage graphique
- Grille de clusters avec couleurs fidèles à l'original :
  - Vert (•) : Cluster défragmenté (Used)
  - Gris (░) : Espace libre (Unused)
  - Blanc (•) : Cluster fragmenté à défragmenter (Pending)
  - Rouge (B) : Bloc défectueux (Bad)
  - Bleu (X) : Bloc système non déplaçable (Unmovable)
  - Jaune (r) : Bloc en lecture (Reading)
  - Vert (W) : Bloc en écriture (Writing)
- Barre de progression
- Statut en temps réel avec temps écoulé

## Architecture du code

### Structures principales

1. `HddSoundGenerator` : Générateur de sons procéduraux pour simuler les bruits de disque dur
   - Types de sons : Seek, Read, Write, Idle
   - Méthodes pour générer différents types de bruits

2. `AudioEngine` : Gestionnaire audio utilisant la bibliothèque `rodio`
   - Lecture de sons pour différentes actions (lecture, écriture, seek)
   - Activation/désactivation du son

3. `Args` : Arguments CLI définis avec `clap`
   - Options pour vitesse, taille de grille, taux de remplissage, activation du son

4. `ClusterState` : Énumération des états possibles d'un cluster
   - Used, Unused, Pending, Bad, Unmovable, Reading, Writing

5. `DefragPhase` : Énumération des phases de la défragmentation
   - Initializing, Analyzing, Defragmenting, Finished

6. `App` : Structure principale de l'application
   - État de l'application (clusters, statistiques, phase)
   - Gestion des menus et dialogues
   - Gestion audio

7. `DefragStats` : Statistiques de la défragmentation
   - Clusters défragmentés, total à défragmenter, temps de début

### Méthodes principales

1. `App::new()` : Initialise l'application avec les paramètres fournis
   - Crée une disposition initiale des clusters aléatoire
   - Configure les paramètres de base

2. `App::run()` : Boucle principale de l'application
   - Gestion des événements clavier
   - Rafraîchissement de l'écran
   - Mise à jour de l'état de l'application

3. `App::update()` : Mise à jour de l'état de l'application
   - Logique de simulation de défragmentation
   - Gestion des différentes phases

4. `App::render()` : Rendu de l'interface complète
   - En-tête avec menus
   - Grille de clusters
   - Pied de page avec statistiques

### Widget graphique

1. `DiskGridWidget` : Widget personnalisé pour afficher la grille de clusters
   - Affiche chaque cluster avec sa couleur correspondante
   - Gère l'affichage fidèle à l'original MS-DOS

## Système audio

Le simulateur inclut un système audio procédural qui génère des bruits de disque dur réalistes :
- Sons de seek (clics mécaniques rapides)
- Sons de lecture (grattement régulier)
- Sons d'écriture (similaire à lecture mais plus intense)
- Ronronnement de fond (idle)

Chaque action de lecture/écriture sur les clusters déclenche un son approprié via le `AudioEngine`.

## Options CLI

- `--speed` : Vitesse d'animation (fast, normal, slow)
- `--size` : Taille de la grille (format WxH, ex. 78x16)
- `--fill` : Pourcentage de remplissage initial du disque
- `-s, --sound` : Activer les sons HDD

## Fonctionnalités de menu

Le simulateur reproduit le comportement des menus MS-DOS :
- Menu Optimize : Begin optimization, Drive..., Optimization Method..., Exit
- Menu Analyze : Analyze drive, File fragmentation...
- Menu File : Print disk map, Save disk map...
- Menu Sort : Différentes options de tri
- Menu Help : Contents, About MS-DOS Defrag...

## Cycle de défragmentation

La simulation suit ce cycle :
1. Trouver un cluster Pending aléatoire
2. Le marquer comme Reading (affiché en jaune)
3. Trouver un cluster Unused disponible
4. Le marquer comme Writing (affiché en vert)
5. Convertir l'ancien cluster Reading en Unused
6. Convertir le cluster Writing en Used
7. Répéter jusqu'à ce que tous les clusters soient défragmentés

## Aspects techniques

1. **Génération procédurale** : Le son est généré en temps réel sans fichiers audio
2. **UI fidèle** : Interface reproduisant exactement l'aspect de MS-DOS Defrag
3. **Animations fluides** : Mise à jour régulière de l'état pour créer une animation
4. **Gestion des interactions** : Système de menus et navigation comme dans l'original
5. **Statistiques en temps réel** : Affichage des progrès et statistiques de défragmentation

## Dépendances

- `ratatui` : Interface utilisateur dans le terminal
- `crossterm` : Gestion des événements clavier et terminal
- `rand` : Génération de nombres aléatoires pour l'initialisation
- `clap` : Gestion des arguments de ligne de commande
- `rodio` : Gestion audio pour les sons procéduraux

## Utilisation

Compiler et exécuter avec `cargo run -- [options]` pour démarrer le simulateur avec les paramètres choisis.