# AIDO (Artificial Intelligence Development Operator) — Der vollständige Produkt-Masterplan

**VERSION 1.0 · 9. NOVEMBER 2026 · PRODUKT- UND ENTWICKLUNGSARCHITEKTUR**

**AIDO = Artificial Intelligence Development Operator**  
Dein persönlicher, intelligenter Desktop-Begleiter.

Ein kleiner, sympathischer, animierter Charakter, der auf deinem Windows-Desktop lebt, deinen Computer kennenlernt, eigenständig mitdenkt und dir Aufgaben abnimmt – mit lokalem KI-Gehirn und dauerhaftem Gedächtnis.

> **„AIDO denkt für dich. Nicht du für AIDO.“**  
> **„Dein PC. Dein AIDO. Deine Daten.“**

---

## 1. Die endgültige Produktvision

Wir entwickeln keine weitere gewöhnliche Chat-Anwendung. AIDO (**Artificial Intelligence Development Operator**) wird eine intelligente Bedienungs-, Organisations- und Automatisierungsschicht für Windows 11.

Der Benutzer muss nicht verstehen, wie KI-Modelle, APIs, Agenten, Docker oder Kommandozeilen funktionieren.

Er installiert AIDO, wählt einige verständliche Einstellungen, erlaubt die gewünschten Zugriffe und lernt anschliessend seinen persönlichen Desktop-Begleiter kennen.

AIDO soll dann:
- **Sehen:** Fenster, Programme und freigegebene Bildschirminhalte verstehen.
- **Hören und sprechen:** Natürliche Gespräche führen und Sprachbefehle entgegennehmen.
- **Handeln:** Programme bedienen, Dateien organisieren und genehmigte Aufgaben erledigen.
- **Lernen:** Arbeitsabläufe, Vorlieben, Projekte und frühere Entscheidungen langfristig kennenlernen.
- **Erinnern:** Informationen intelligent miteinander verknüpfen und nur relevanten Kontext laden.
- **Vorausdenken:** Probleme erkennen, Ideen entwickeln und selbstständig Aufgaben vorschlagen.
- **Organisieren:** Anwendungen, Projekte, Aufgaben und Agenten koordinieren.
- **Schützen:** Persönliche Informationen lokal verarbeiten und externe Datenübertragungen kontrollieren.

### Eine wichtige technische Präzisierung
Deine Aussage „AIDO kann alles, was der Nutzer auch kann, nur 100-mal schneller und besser“ ist als Produktvision hervorragend.

Technisch müssen wir unterscheiden: Viele digitale Aufgaben kann AIDO tatsächlich schneller erledigen, einige genauso gut, andere zunächst weniger zuverlässig. Anwendungen mit speziellen Sicherheitsmechanismen, CAPTCHAs, geschützten Anmeldefenstern oder nicht zugänglichen Funktionen können Grenzen setzen.

Unser Entwicklungsziel lautet deshalb:
> **AIDO soll alle technisch zugänglichen Benutzerhandlungen verstehen, möglichst zuverlässig automatisieren und bei Unsicherheit verständlich um Hilfe oder Freigabe bitten.**

Bei den Berechtigungen steht das Konzept jetzt fest: Full Access ist möglich, aber relevante Änderungen benötigen eine nachvollziehbare Freigabe. Diese Schutzregel wird nicht allein dem KI-Modell überlassen.

---

## 2. Die festgelegten Produkteigenschaften

| Bereich | Festlegung |
| :--- | :--- |
| **Name & Bedeutung** | **AIDO** — *Artificial Intelligence Development Operator* |
| **Zielgruppe** | Jeder Windows-Nutzer, auch ohne technische Kenntnisse |
| **Betriebssystem** | Windows 11, Home und Pro soweit technisch unterstützt |
| **Oberfläche** | Animierter, beweglicher 3D-Desktop-Charakter |
| **Hauptintelligenz** | Lokal betriebenes Sprach-/Vision-Modell |
| **Datenschutz** | Private Offline, Local Connected und Hybrid AI |
| **Berechtigungen** | Standard Access und Full Access |
| **Verhalten** | Selbstlernend, proaktiv, mit eigenständiger Aufgabenplanung |
| **Apps** | Automatisch erkennen, auswählen, verbinden und testen |
| **Adapter** | Vorhandene Schnittstellen bevorzugen, neue kontrolliert entwickeln |
| **Gedächtnis** | Dauerhaft, strukturiert, durchsuchbar und automatisch komprimiert |
| **Systemverwaltung** | Analyse, Optimierung, Cleanup und Projektorganisation |
| **Hintergrundarbeit** | Dauerhafte Aufgaben und Automatisierungen mit Freigabegrenzen |
| **Entwicklung** | Zunächst dein Predator-PC als Referenzsystem |
| **Veröffentlichung** | Installierbares Windows-Produkt, später erweiterbar |

Das Produkt soll ohne Cloud-Konto benutzbar sein, sofern der PC ein geeignetes lokales Modell ausführen kann.

Damit erhalten wir ein klares Alleinstellungsversprechen: AIDO verbindet lokale KI, persönliche Langzeiterinnerungen, proaktive Unterstützung und echte Desktop-Bedienung in einer Oberfläche, die sich nicht wie ein Entwicklerwerkzeug anfühlt.

---

## 3. Neue Erkenntnisse aus der technischen Recherche

Bei der aktuellen Überprüfung der verfügbaren Technologien sind vier Ergebnisse für die Architektur besonders relevant:

1. **Microsoft WinApp CLI — Favorit für Desktop-Steuerung:**
   `winapp ui` kann Fenster untersuchen, Bedienelemente finden, Werte auslesen, Aktionen ausführen und Screenshots erstellen. Neuere Versionen unterstützen auch automatisierte Tests in Windows Sandbox.
2. **Hermes kann Teile des KI-Managements übernehmen:**
   Lokale Modellverwaltung, bedarfsgerechte Kontextvergrösserung, Komprimierung und automatisches Entladen ungenutzter Modelle sowie Hintergrundaufgaben.
3. **Mem0 ist lokal nutzbar, aber nicht automatisch offline:**
   Für AIDOs Private-Offline-Modus müssen alle Memory-Komponenten explizit lokal konfiguriert und Netzwerkübertragungen getestet werden.
4. **Qwen3.8-27B als Hauptmodell-Kandidat:**
   Unterstützt Bilder und Videos, nativer Kontext bis 262.144 Tokens. Auf dem 32-GB-VRAM-Referenz-PC nutzen wir `Qwen3.8-27B-UD-Q4_K_M.gguf` samt `mmproj-Qwen3.8-27B-BF16.gguf` und `Qwen3.8-27B-DFlash2-Q4_K_M.gguf`.

---

## 4. Die vollständige AIDO-Systemarchitektur

- **AIDO Desktop:** 3D-Charakter · Chat · Voice · Gesten · Screenshots · Einstellungen
- **AIDO Brain:** Lokales Hauptmodell · Hermes · Planung · Subagenten · Proaktivität
  - **Memory Engine:** Fakten, Verlauf, Beziehungen und Projekte
  - **App Intelligence:** App-Erkennung, Adapter und Skills
  - **Task Engine:** Aufgabenplanung und Hintergrundarbeit
  - **Project Intelligence:** Projektindex und Arbeitsumgebungen
- **Permission Gateway:** Berechtigungen · Freigaben · Datenschutz · Protokollierung
- **Windows Bridge:** Programme, Fenster, Dateien, Prozesse
- **Admin Broker:** Gezielte administrative Funktionen

### Geplanter Technologie-Stack

| Komponente | Bevorzugte Lösung |
| :--- | :--- |
| **Desktop-App** | Tauri 2, Rust, TypeScript |
| **Charakter** | Three.js, glTF/VRM, später eigenes 3D-Modell |
| **Windows-Steuerung** | Microsoft WinApp CLI, Windows UI Automation |
| **KI-Orchestrierung** | Hermes als Kernkandidat, hinter eigener AIDO-Schnittstelle |
| **Hauptmodell** | Qwen3.8-27B für den Predator |
| **KI-Runtime** | `llama.cpp` (`llama-server.exe` CUDA 13.4 mit Idle-Sleep & Autoload) |
| **Memory-Datenbank** | SQLite mit FTS5, ergänzend lokales semantisches Memory |
| **Memory-Erweiterung** | Mem0; Wissensgraph bei nachgewiesenem Mehrwert |
| **App-Adapter** | API, CLI, MCP, UI Automation, Vision |
| **Automatisierung** | Persistente Task Queue und ereignisgesteuerte Planung |
| **Datenschutz** | Kontrollierte Netzwerkzugriffe und lokale Secret-Verwaltung |
| **Isolation** | Windows Sandbox, VM und eingeschränkte Testprozesse |

---

## 5. AIDO Memory — das wichtigste Langzeitsystem

### 5.1 Die sieben Gedächtnisebenen

| Ebene | Inhalt | Speicherstrategie |
| :--- | :--- | :--- |
| **1. SOUL Memory** | Persönlichkeit, Rolle, Grundregeln, Verhalten | Klein, bei jeder Anfrage verfügbar |
| **2. User Memory** | Vorlieben, Ziele, Arbeitsweise | Dauerhaft, kontrolliert aktualisiert |
| **3. System Memory** | Hardware, Apps, Pfade, Konfigurationen | Strukturierte Datenbank |
| **4. Project Memory** | Projektstatus, Entscheidungen, Dateien, Aufgaben | Eigenständige Projektbereiche |
| **5. Episodic Memory** | Gespräche, Ereignisse, ausgeführte Aktionen | Chronologisch, durchsuchbar |
| **6. Semantic Memory** | Erkenntnisse und Beziehungen | Vektorsuche, optional Wissensgraph |
| **7. Procedural Memory** | Gelernte Arbeitsabläufe und wiederverwendbare Skills | Versionierte Anleitungen und Adapter |

### 5.2 Der automatische Lernprozess
1. **Ereignis erkennen:** Gespräch, Programmnutzung, Projektänderung, erfolgreich ausgeführte Aufgabe.
2. **Erkenntnisse extrahieren:** Was wurde gelernt? Ist es wichtig, überprüfbar und für später nützlich?
3. **Mit bestehendem Wissen abgleichen:** Duplikate, Widersprüche und veraltete Fakten erkennen.
4. **Verknüpfen und speichern:** Nutzer ↔ Projekt ↔ App ↔ Datei ↔ Entscheidung ↔ Aufgabe.
5. **Wissen bei Bedarf abrufen:** Nur die Informationen in den Modellkontext laden, die für die aktuelle Aufgabe erforderlich sind.

### 5.3 Intelligente Kontextkomprimierung
- **Nach wichtigen Entscheidungen:** Fakten und Beschlüsse speichern.
- **Nach erfolgreicher Aufgabe:** Ergebnis und wiederverwendbaren Ablauf dokumentieren.
- **Bei wachsendem Gesprächskontext:** Ältere Inhalte zusammenfassen.
- **Beim Projektwechsel:** Aktiven Kontext auf das neue Projekt umstellen.
- **Während Leerlaufzeiten:** Duplikate erkennen und Erinnerungen konsolidieren.
- **Bei erneuter Nutzung:** Relevantes Wissen aus der lokalen Datenbank abrufen.
- **Bei veränderten Systemzuständen:** Veraltete Einträge kennzeichnen oder ersetzen.

---

## 6. Die AIDO-Intelligenz: Ein Hauptassistent, mehrere Spezialisten

| Interner Spezialist | Verantwortung |
| :--- | :--- |
| **AIDO Core** | Versteht Nutzerabsichten, plant und koordiniert |
| **System Expert** | Windows, Hardware, Ressourcen und Fehleranalyse |
| **App Expert** | Anwendungen erkennen, verbinden und bedienen |
| **Project Expert** | Projekte, Dateien, Git-Repositories und Entwicklungsumgebungen |
| **Memory Curator** | Erinnerungen aktualisieren und zusammenfassen |
| **Automation Expert** | Hintergrundaufgaben und wiederkehrende Abläufe |
| **Security Expert** | Aktionen gegen Berechtigungsregeln prüfen |
| **Research Expert** | Bei erlaubtem Internetzugriff Informationen recherchieren |

### 6.1 Das schlafende Hauptmodell
Zyklus: **Schlafen → Denken → Arbeiten → Entladen**.  
Im Schlafmodus bleiben die leichte Desktop-Anwendung, Ereignisüberwachung und Aufgabenverwaltung aktiv. Das grosse Modell gibt seinen VRAM nach Leerlauf frei (`--sleep-idle-seconds`) und wacht bei Bedarf automatisch wieder auf (`--models-autoload`).

---

## 7. Smart Installer — vom Download bis zum Erwachen

1. **Willkommen & Datenschutz:** Private Offline, Local Connected oder Hybrid AI.
2. **Hardware automatisch erkennen:** Windows, RAM, GPU, Treiber und bereits vorhandene KI-Runtimes untersuchen.
3. **Berechtigungsmodus wählen:** Standard Access oder Full Access.
4. **KI automatisch einrichten:** Vorhandenes Modell wiederverwenden oder passendes Modell vorschlagen.
5. **Apps entdecken:** Installierte Programme erkennen und auswählbare Übersicht anzeigen.
6. **Apps intelligent integrieren:** Passende Adapter einrichten, Verbindungen autorisieren und Funktionen testen.
7. **Wissen aufbauen:** Optional freigegebene Ordner, Projekte, Dokumente und bestehende Chats analysieren.
8. **AIDO validieren:** KI, Memory, Desktop-Steuerung und Verbindungen testen.
9. **AIDO erwacht:** Animierter Charakter erscheint und beginnt das erste Gespräch.

---

## 8. App Intelligence — AIDO erkennt und beherrscht Programme

Integrationskaskade:
**Offizielle API → vorhandene CLI → geprüfter MCP-/Plugin-Adapter → neu entwickelter Adapter → Windows UI Automation → visuelle Steuerung.**

---

## 9. Proactive Engine — AIDO denkt selbstständig voraus

- **Beobachten:** Ein freigegebenes Projekt wurde verändert.
- **Verstehen:** Änderungen sind vorhanden, aber noch nicht gesichert.
- **Vorausdenken:** Eine Sicherung wäre sinnvoll.
- **Vorschlagen:** „Soll ich eine Projektsicherung vorbereiten?“

---

## 10. Desktop-Charakter — AIDOs sichtbare Persönlichkeit

- **Klick:** Chat öffnen
- **Gedrückt halten:** Sprachnachricht aufnehmen (Push-to-Talk)
- **Mittelklick:** Screenshot als Chatkontext
- **Rechtsklick:** Funktionen und Einstellungen (Systemanalyse, Fast/Deep Cleanup, Privacy Center, Memory & Wissen, Schlafen legen)

---

## 11. Privacy Center und Sicherheitsarchitektur

| Modus | KI-Verarbeitung | Externe Datenübertragung durch AIDO |
| :--- | :--- | :--- |
| **Private Offline** | Vollständig lokal | Gesperrt (nur `127.0.0.1`) |
| **Local Connected** | Lokal | Nur für erlaubte Dienste und Funktionen |
| **Hybrid AI** | Lokal und optional Cloud | Vorher definierte Übertragungen |

---

## 12. Geplante zentrale Datenorganisation

| Ort | Zweck |
| :--- | :--- |
| `%LOCALAPPDATA%\AIDO` | Benutzerkonfiguration und lokale AIDO-Daten |
| `%LOCALAPPDATA%\AIDO\data` | Memory, Aufgaben und Projektindex |
| `%LOCALAPPDATA%\AIDO\adapters` | App-Integrationen |
| `%PROGRAMDATA%\AIDO` | Geschützte Dienste bei systemweiter Installation |
| Bestehende Projektordner | Werden zunächst nur referenziert |
| Bestehende Modellordner | Werden wiederverwendet |

---

## 13. Entwicklungsroadmap — die drei Meilensteine

- **Meilenstein A — AIDO funktioniert (Priorität 1):**
  - Phase 0: Bestand sichern und Architektur festlegen
  - Phase 1: AIDO Brain (Qwen3.8-27B + Vision + Whisper ASR + Idle-Sleep)
  - Phase 2: Windows Bridge + Permission Gateway
  - Phase 3: Memory v1 (SQLite + FTS5)
- **Meilenstein B — AIDO wird zum Produkt (Priorität 2):**
  - Phase 4: Desktop-Charakter
  - Phase 5: Smart Installer
  - Phase 6: App Intelligence (Capability Registry & Adapter)
  - Phase 7: Project & System Expert (Projektkatalog + Fast/Deep Cleanup)
- **Meilenstein C — AIDO denkt voraus (Priorität 3):**
  - Phase 8: Proactive Engine
  - Phase 9: Langzeitautomatisierung
  - Phase 10: Produktqualität und Sicherheit
  - Phase 11: Geschlossene Beta
