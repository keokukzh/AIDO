# Entwicklungsroadmap

AIDO soll sich per Windows-Installer einrichten lassen, als MONO auf dem Desktop erwachen und mit freigegebenen Projekten und Werkzeugen arbeiten. Bei Aktionen zeigt MONO das konkrete Ziel und die Wirkung. Der Nutzer kann zustimmen, ablehnen oder den Vorschlag ändern. Der [Projektstatus](PROJECT_STATUS.md) hält fest, welche Abläufe im installierten Build geprüft sind.

## 1. Installierten Desktop-Meilenstein abnehmen (Abgeschlossen)

Auf dem Referenz-PC wurde AIDO 0.1.2 erfolgreich installiert. Unterbrochenes Setup, Upgrade, Projektauswahl, SQLite-Persistenz und Werkzeugstatus wurden verifiziert. REA 6.7.0 und die lesende PE-Ressourcenanalyse auf `C:\Windows\System32\notepad.exe` sind bestätigt. Für «Öffne mein Projekt AIDO» öffnet Antigravity exakt `C:\Users\keoku\Desktop\AIDO`. Ablehnen, Ändern, Einmal-Tokens, Replay-Schutz und Audit-Logging wurden abgenommen. Blender-Start dient als bestätigter Regressionstest.

Native Windows UI Automation auf Basis von Microsoft UIA und `vercel-labs/agent-desktop` v1.2.0 wurde vollständig in `aido-core` integriert (`ui_automation.rs`) und im 72,34-MiB-Installer mitgeliefert. Interaktive Bedienelemente werden über semantische Snapshots (`@ref`) zielgenau bedient und im `audit_log` protokolliert.

Qwen in WSL2 wurde durch `--load-mode none` von Kernel-mmap-Hängern befreit und antwortet mit `391` und `392` in 45,12 s bei voller VRAM-Freigabe nach Shutdown. Hermes blieb auf Port 18434 ungestört aktiv. Der Desktop-Start wurde durch Win32-Thread-Desktop-Kopplung an `WinSta0\Default` gehärtet.

## 2. Sprache, Screenshot und MONO-Bedienung

Grösse und Position speichern. Gedrückthalten nimmt Audio für lokales Whisper auf. Mittelklick hängt einen sichtbar markierten Screenshot an die nächste Nachricht an. Aufnehmen, Arbeiten, Freigabe und Fehler erhalten unterscheidbare Zustände. Abnahme mit tatsächlich gesprochener Eingabe, sichtbarem Anhang und nachvollziehbarer Vision-Antwort. Die Modellpfade und Messungen stehen in der [Laufzeitübersicht](MODEL_RUNTIME_INVENTORY.md).

## 3. Gedächtnis, proaktive Hilfe und Aufgaben (Abgeschlossen)

SQLite/FTS5 ist an den nativen Desktop-Chat und das neue Privacy & Memory Center (`memory.html`) angebunden. Sitzungsverlauf, Vorlieben und Projektwissen werden aktiv einsehbar, suchbar und über die Schaltfläche «Vergessen» sofort löschbar verwaltet. Im Chat werden relevante Fakten automatisch in den Kontext eingespeist und neue Nutzerpräferenzen selbstständig gelernt.

Die proaktive MONO-Engine blendet bei ungesicherten Git-Änderungen in freigegebenen Projekten eine sanfte Sprechblase über MONO ein (Optionen: Ansehen, Später, Ausblenden). Ruhezeiten und Vollbild/Gaming werden über den Fokus-Guard strikt geschützt. Der Wochenbericht als ausführbarer Skill baut auf dieser Datenbasis auf.

## 4. Aktionen, Skills und Integrationen (Kern abgeschlossen)

Geprüfte Programmstarts und App-Funktionen sind über fest definierte Aktionen angebunden. Blender 5.1 wird über den neuen Adapter (`blender_connector.rs`) im Headless-Modus für Szenenanalysen, Python-Skripte und Rendering mit Einmal-Token-Freigabe gesteuert. Docker Desktop und die drei WSL2-Power-User-Sandboxes sind über `docker_controller.rs` kontrollierbar, während Docker für Normalanwender optional bleibt.

Für sensible Zugangsdaten und Tokens ist Windows-DPAPI (`CryptProtectData` / `CryptUnprotectData`) direkt im Windows-Kern integriert. Der UAC- und Port-Monitor schützt Hermes auf Port 18434 und isoliert AIDO im Standard-Integritätslevel. Das lokale Whisper.cpp-Fundament (CUDA 13.4, `ggml-small.bin`) ist am Referenzpfad verifiziert und alle 106 Tests der Testsuite sind grün.

Weitere optionale Erweiterungen: Android-Cockpit (ADB-Bridge / Notification-Sync) und externe Cloud-Connectors (GitHub, Google Drive) nach Bedarf.

## 5. Windows-Produkt veröffentlichen

Installer auf einem sauberen Windows-11-System ohne Entwicklungswerkzeuge und auf einem schwächeren Rechner testen. Signierten Installer, Updates und Deinstallation abnehmen. Projekte, Modelle und lokale Nutzerdaten bei Upgrade und Deinstallation erhalten und die Wiederherstellung prüfen. Offlinebetrieb und niedrige Hardware-Ausstattung in die reproduzierbare Release-Prüfung aufnehmen.

## Bisherige Grundlagen

Die Phasen 0–13 lieferten Rust-Kernmodule, CLI-Demos und die Browser-Oberfläche. Dazu gehören Freigaben, Projektkatalog, Gedächtnis, Bereinigung, Aufgaben, Spezialisten, REA-Prototyp, Android-ADB und Agent-Remote-Entwürfe. Die Browser-Vorschau in Phase 14 ergänzt MONO. Der [Masterplan](MASTERPLAN.md) beschreibt die Produktidee und der [Projektstatus](PROJECT_STATUS.md) die native Abnahme.
