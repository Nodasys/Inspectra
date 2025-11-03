# 🚀 Guide de Compilation Inspectra

## 📋 Prérequis

Vous avez déjà installé avec succès :
- ✅ Rust (cargo)
- ✅ Tauri CLI v1.6.6

## 🎯 Compilation Rapide (3 étapes)

### 1️⃣ Tester le Core
```powershell
# Vérifier que le core compile correctement
cd c:\Users\Admin\Documents\GitHub\Inspectra
cargo build
```

### 2️⃣ Lancer l'Application (Mode Développement)
```powershell
# Lancer l'interface graphique en mode dev
cd inspectra-gui
cargo tauri dev
```

Cela va :
- Compiler le backend Rust
- Lancer un serveur de développement
- Ouvrir l'application dans une fenêtre

### 3️⃣ Créer l'Installateur (Mode Production)
```powershell
cd inspectra-gui
cargo tauri build
```

L'installateur sera créé dans :
```
inspectra-gui\src-tauri\target\release\bundle\msi\
```

Vous trouverez :
- `Inspectra_0.1.0_x64_en-US.msi` - Installateur Windows
- Ou dans `release\` : `inspectra.exe` - Exécutable standalone

## 🎨 Fonctionnalités de l'Application

### Interface Graphique Complète
✅ **Navigateur de Processus**
- Liste tous les processus Windows en cours d'exécution
- Recherche en temps réel par nom
- Affichage PID, architecture (x64/x86)

✅ **Scanner Mémoire**
- Types supportés : Int32, Int64, Float32, Float64, String
- Scan initial + filtrage des résultats
- Limite intelligente (1000 résultats max)

✅ **Visualisation des Résultats**
- Table avec adresses hexadécimales
- Valeurs formatées
- Copie rapide des adresses

✅ **Gestion d'État**
- Indication du processus attaché
- Barre de statut en temps réel
- Gestion d'erreurs

## 🛠️ Structure du Projet

```
Inspectra/
├── core/                  # Engine Rust (scanner, memory, process)
├── bindings/
│   └── python/           # Bindings PyO3
├── inspectra-gui/        # Application Desktop
│   ├── src/
│   │   └── index.html    # Frontend (HTML/CSS/JS)
│   └── src-tauri/
│       ├── src/
│       │   └── main.rs   # Backend Rust + Tauri commands
│       └── Cargo.toml
└── Cargo.toml            # Workspace
```

## 🐛 Résolution de Problèmes

### Erreur : "VCRUNTIME140.dll manquant"
Installez Visual C++ Redistributable:
https://aka.ms/vs/17/release/vc_redist.x64.exe

### Erreur : "Permission denied"
L'application nécessite des droits administrateur pour accéder à la mémoire des processus.

### Antivirus bloque l'exécution
C'est un faux positif. Les outils de memory analysis sont souvent détectés comme "hacktool".
Ajoutez une exception pour `inspectra.exe`.

### Build lent la première fois
Normal ! La compilation de toutes les dépendances Rust prend 5-10 minutes la première fois.
Les builds suivants seront beaucoup plus rapides (< 1 minute).

## 📦 Distribution

### Fichiers à distribuer :
1. **Pour utilisateurs normaux** : Le fichier MSI
   - Taille : ~5-8 MB
   - Installation propre dans Program Files
   - Désinstallation via Windows

2. **Pour développeurs** : L'exécutable .exe
   - Portable, pas d'installation
   - Peut être copié n'importe où

### Signer l'application (optionnel)
Pour éviter les avertissements Windows SmartScreen, vous pouvez :
1. Obtenir un certificat de signature de code
2. Utiliser `signtool` pour signer le MSI/EXE

## 🎓 Prochaines Étapes

Après le build :
1. ✅ Testez l'application avec un processus simple (notepad.exe)
2. ✅ Essayez un scan Int32 avec une valeur connue
3. ✅ Vérifiez que les résultats s'affichent correctement
4. ✅ Testez la recherche de processus

## 🔐 Sécurité

⚠️ **Important** :
- Cette application nécessite des privilèges élevés
- Elle peut lire/écrire dans la mémoire d'autres processus
- Utilisez-la de manière responsable et légale
- Ne l'utilisez que sur vos propres applications ou avec permission

## 📞 Support

Si vous rencontrez des erreurs :

1. **Vérifier les versions** :
```powershell
cargo --version    # Devrait être 1.70+
cargo tauri --version  # v1.6.6
```

2. **Build en mode verbose** :
```powershell
cargo tauri build --verbose
```

3. **Nettoyer et rebuild** :
```powershell
cargo clean
cargo build
```

---

## 🎉 Résultat Final

Vous aurez :
- ✅ Une application Windows native professionnelle
- ✅ Interface graphique moderne (dark theme)
- ✅ Performance optimale (Rust + Tauri)
- ✅ Petit fichier (~5-8 MB vs 100+ MB pour Electron)
- ✅ Installateur MSI professionnel

**Prêt à compiler ? Lancez `cargo tauri dev` ! 🚀**
