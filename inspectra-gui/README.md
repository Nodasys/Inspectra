# 🖥️ Inspectra GUI - Application Desktop

## ✅ APPLICATION COMPLÈTE ET FONCTIONNELLE !

Une application desktop **moderne** pour l'analyse mémoire en temps réel avec Tauri + Rust.

---

## 🚀 Démarrage Rapide

### Mode Développement (Test)
```powershell
cd inspectra-gui
cargo tauri dev
```

### Build Production (Créer l'installateur)
```powershell
cd inspectra-gui
cargo tauri build
```

📦 **L'installateur MSI sera dans** : `src-tauri\target\release\bundle\msi\`

---

## 🎨 Interface & Fonctionnalités

### Design Moderne
✅ Theme sombre professionnel  
✅ Interface intuitive à 2 panneaux  
✅ Responsive et performante  
✅ Style Windows 11

### Fonctionnalités

#### 📋 Navigateur de Processus
- Liste tous les processus Windows
- Recherche en temps réel
- Détails : PID, architecture, chemin

#### 💾 Scanner Mémoire
**Types supportés :** Int32, Int64, Float32, Float64, String  
**Capacités :** Scan multi-threadé, limite 1000 résultats

#### 📊 Résultats
- Table avec adresses hex
- Copie rapide au clipboard
- Affichage formaté

---

## ⚡ Performance & Avantages

**Tauri vs Electron:**
- 🔥 95% plus léger (~5 MB vs ~100 MB)
- ⚡ Moins de RAM (WebView natif)
- 🚀 Plus rapide (pas de Node.js)
- 🔒 Plus sécurisé

---

# Build release (creates installer)
cargo tauri build
```

### Distribution

After building, installers will be in:
- `src-tauri/target/release/bundle/`

Windows installer: `*.msi` or `*.exe`

## Features

- 🔍 Process browser and search
- 🎯 Attach to any process
- 💾 Memory scanner (Int32, Int64, Float32, Float64, String)
- 📊 Real-time results display
- 🎨 Modern dark theme UI
- ⚡ Native performance

## Usage

1. Launch the application
2. Click "Refresh Processes" to load running processes
3. Search and select a target process
4. Enter a value and click "Scan Memory"
5. View results and copy addresses

## Keyboard Shortcuts

- `Ctrl+R` - Refresh processes
- `Ctrl+F` - Focus search
- `Ctrl+Q` - Quit application
