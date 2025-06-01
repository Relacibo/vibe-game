# Vibe Game

**Vibe Game** ist ein experimentelles 3D-Spiel in Rust mit Bevy, das prozedural generierte Bäume, Physik und eine offene Welt bietet. Die Assets werden automatisiert erzeugt und als GLB importiert.

---

## Was ist möglich?

- **Prozedurale Bäume:** Jeder Baum ist einzigartig (Stammdicke, Kronenform, Äste, Texturen, Bumpmaps).
- **Bewegung:** Du steuerst einen Spieler mit WASD und Maus durch die Welt.
- **Leben:** Der Spieler hat Lebenspunkte (Health). Kollisionen mit bestimmten Objekten (z.B. Projektilen oder Hindernissen) verringern das Leben, Heilobjekte können es wieder auffüllen.
- **Physik:** Es gibt Kollisionen, Gravitation und Sprungmechanik.
- **Echtzeit-Licht:** Alle Licht- und Schatteneffekte werden in der Engine berechnet.
- **Web-Version:** Das Spiel läuft auch als WebAssembly im Browser.

---

## Steuerung

- **WASD** – Bewegung
- **Maus** – Kamera drehen
- **Leertaste** – Springen (falls aktiviert)
- **ESC** – Beenden

---

## Web-Version

Du kannst das Spiel direkt im Browser ausprobieren:  
👉 **[https://relacibo.github.io/vibe-game/](https://relacibo.github.io/vibe-game/)**

---

## Asset- und Skript-Workflow

Alle Details zur Baum- und Textur-Generierung findest du im [scripts/README.md](scripts/README.md).

---

## Lizenz

Dieses Projekt steht unter der [MIT-Lizenz](LICENSE).

---

## Generierungshinweis

> **Hinweis:**  
> Der Großteil dieses Readmes sowie der Skripte und ein erheblicher Teil des Codes wurden mit Unterstützung von GitHub Copilot (GPT-4.1) unter Anleitung von Relacibo (dem Projektinhaber) generiert.  
>  
> _Mit ❤️ von GitHub Copilot/GPT-4.1 unter Anweisung von Relacibo._
