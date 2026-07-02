//! The idle Weave (mockups/01-desktop-idle.html, made real): greeting on the
//! Stage, a live rail, the Intent Bar with a working caret, and the presence
//! dot breathing beside it. Elements are laid out disjoint so damage-driven
//! repaints can clear-and-redraw per element.

use crate::fb::{Frame, Rect, INK, INK_DIM, INK_FAINT, PRESENCE};
use crate::font::Text;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const BREATH_SECS: f32 = 3.2;
const THINK_SECS: f32 = 1.1;
const CARET_BLINK: Duration = Duration::from_millis(530);
const MATERIALIZE: Duration = Duration::from_millis(520);
/// A service is "alive" if the bus heard from it this recently (heartbeats
/// come every 5s).
const ALIVE_WINDOW: Duration = Duration::from_secs(12);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Element {
    Greeting,
    Rail,
    IntentBar,
    Presence,
}

pub struct Scene {
    start: Instant,
    pub typed: String,
    caret_on: bool,
    last_blink: Instant,
    thinking_until: Instant,
    services: HashMap<String, Instant>,
    alive_shown: usize,
    // layout
    greeting: Rect,
    rail: Rect,
    bar: Rect,
    presence: Rect,
    presence_c: (f32, f32),
    greeting_line: String,
}

impl Scene {
    pub fn new(w: i32, h: i32) -> Self {
        let margin = 26;
        let rail_w = 300;
        let stage_w = w - rail_w - margin * 3;

        let rail = Rect::new(w - rail_w - margin, 76, rail_w, 210);
        let greeting = Rect::new(
            margin + stage_w / 2 - 280,
            (h as f32 * 0.30) as i32,
            560,
            120,
        );
        let bar_w = 560.min(stage_w - 120);
        let bar = Rect::new(margin + (stage_w - bar_w) / 2, h - 78, bar_w, 46);
        // The presence sits left of the bar, far enough that its glow rect
        // stays disjoint.
        let pr = 26;
        let presence = Rect::new(bar.x - 2 * pr - 22, bar.y + bar.h / 2 - pr, pr * 2, pr * 2);

        let hour = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
            / 3600)
            % 24;
        let greeting_line = match hour {
            5..=11 => "Good morning.",
            12..=17 => "Good afternoon.",
            _ => "Good evening.",
        }
        .to_string();

        Self {
            start: Instant::now(),
            typed: String::new(),
            caret_on: true,
            last_blink: Instant::now(),
            thinking_until: Instant::now(),
            services: HashMap::new(),
            alive_shown: usize::MAX, // force first rail paint
            greeting,
            rail,
            bar,
            presence,
            presence_c: (presence.x as f32 + pr as f32, presence.y as f32 + pr as f32),
            greeting_line,
        }
    }

    pub fn note_service(&mut self, name: String) {
        self.services.insert(name, Instant::now());
    }

    pub fn key_char(&mut self, c: char) -> Element {
        if self.typed.len() < 96 {
            self.typed.push(c);
        }
        Element::IntentBar
    }

    pub fn key_backspace(&mut self) -> Element {
        self.typed.pop();
        Element::IntentBar
    }

    /// Enter: the Cortex arrives at M6; for now the Weave acknowledges by
    /// thinking for a moment. Returns the spoken intent for the log.
    pub fn key_enter(&mut self) -> (String, [Element; 2]) {
        let said = std::mem::take(&mut self.typed);
        self.thinking_until = Instant::now() + Duration::from_secs_f32(1.4);
        (said, [Element::IntentBar, Element::Presence])
    }

    /// Advance time; returns which elements need repainting this frame.
    pub fn tick(&mut self) -> Vec<Element> {
        let mut damage = vec![];
        let t = self.start.elapsed();

        // Materialize: everything animates in during the first moments.
        if t < MATERIALIZE + Duration::from_millis(400) {
            damage.extend([
                Element::Greeting,
                Element::Rail,
                Element::IntentBar,
                Element::Presence,
            ]);
            return damage;
        }

        // The presence breathes continuously.
        damage.push(Element::Presence);

        // Caret blink.
        if self.last_blink.elapsed() >= CARET_BLINK {
            self.caret_on = !self.caret_on;
            self.last_blink = Instant::now();
            damage.push(Element::IntentBar);
        }

        // Rail repaints only when the live service count changes.
        let now = Instant::now();
        let alive = self
            .services
            .values()
            .filter(|seen| now.duration_since(**seen) < ALIVE_WINDOW)
            .count();
        if alive != self.alive_shown {
            self.alive_shown = alive;
            damage.push(Element::Rail);
        }

        damage.dedup();
        damage
    }

    pub fn rect_of(&self, e: Element) -> Rect {
        match e {
            Element::Greeting => self.greeting,
            Element::Rail => self.rail,
            Element::IntentBar => self.bar,
            Element::Presence => self.presence,
        }
        .inflate(2)
    }

    /// Materialize easing for an element (staggered fade + rise).
    fn enter(&self, e: Element) -> (f32, f32) {
        let delay = match e {
            Element::Greeting => 0.0,
            Element::Rail => 0.10,
            Element::IntentBar | Element::Presence => 0.20,
        };
        let t = (self.start.elapsed().as_secs_f32() - delay) / MATERIALIZE.as_secs_f32();
        let p = t.clamp(0.0, 1.0);
        let ease = p * (2.0 - p); // ease-out
        (ease, 8.0 * (1.0 - ease)) // (alpha, rise)
    }

    pub fn draw(&mut self, f: &mut Frame, text: &mut Text, e: Element) {
        f.clear_region(self.rect_of(e));
        match e {
            Element::Greeting => self.draw_greeting(f, text),
            Element::Rail => self.draw_rail(f, text),
            Element::IntentBar => self.draw_bar(f, text),
            Element::Presence => self.draw_presence(f),
        }
    }

    fn draw_greeting(&mut self, f: &mut Frame, text: &mut Text) {
        let (a, rise) = self.enter(Element::Greeting);
        if a <= 0.0 {
            return;
        }
        let r = self.greeting;
        let title_px = 30.0;
        let w = text.width(&self.greeting_line, title_px);
        let x = r.x + (r.w - w) / 2;
        let y = r.y + rise as i32;
        text.draw(f, &self.greeting_line, x, y, title_px, INK, a);

        let sub = "The Weave is awake. The mind arrives at M6 - for now, this is home.";
        let sw = text.width(sub, 14.0);
        text.draw(f, sub, r.x + (r.w - sw) / 2, y + 46, 14.0, INK_DIM, a);
    }

    fn draw_rail(&mut self, f: &mut Frame, text: &mut Text) {
        let (a, rise) = self.enter(Element::Rail);
        if a <= 0.0 {
            return;
        }
        let r = self.rail;
        let y = r.y + rise as i32;
        text.draw(f, "THIS MACHINE", r.x + 6, y, 11.0, INK_FAINT, a);

        let card1 = Rect::new(r.x, y + 24, r.w, 74);
        f.glass(card1, 16.0, 0.045 * a, 0.09 * a);
        text.draw(f, "mind plane", card1.x + 14, card1.y + 14, 14.0, INK, a);
        let alive = if self.alive_shown == usize::MAX {
            0
        } else {
            self.alive_shown
        };
        let line = format!("{alive} services alive on the bus");
        text.draw(f, &line, card1.x + 14, card1.y + 38, 12.5, INK_DIM, a);

        let card2 = Rect::new(r.x, card1.y + 74 + 14, r.w, 74);
        f.glass(card2, 16.0, 0.045 * a, 0.09 * a);
        text.draw(f, "substrate", card2.x + 14, card2.y + 14, 14.0, INK, a);
        text.draw(
            f,
            "wakes at M3 - nothing indexed yet",
            card2.x + 14,
            card2.y + 38,
            12.5,
            INK_DIM,
            a,
        );
    }

    fn draw_bar(&mut self, f: &mut Frame, text: &mut Text) {
        let (a, rise) = self.enter(Element::IntentBar);
        if a <= 0.0 {
            return;
        }
        let mut r = self.bar;
        r.y += rise as i32;
        f.glass(r, (r.h / 2) as f32, 0.05 * a, 0.10 * a);

        let pad = 20;
        let ty = r.y + (r.h - 18) / 2;
        let (shown, color, alpha) = if self.typed.is_empty() {
            (
                "ask the computer - the Cortex wakes at M6",
                INK_FAINT,
                0.9 * a,
            )
        } else {
            (self.typed.as_str(), INK, a)
        };
        let tw = text.draw(f, shown, r.x + pad, ty, 14.0, color, alpha);

        // Caret after the typed text (steady while the field is empty too).
        if self.caret_on && a >= 1.0 {
            let cx = r.x + pad + if self.typed.is_empty() { 0 } else { tw + 2 };
            for y in ty..ty + 18 {
                f.blend(cx, y, PRESENCE, 0.85);
                f.blend(cx + 1, y, PRESENCE, 0.85);
            }
        }

        // Dial chip, right-aligned inside the bar.
        let chip = "dial: Balanced";
        let cw = text.width(chip, 11.0);
        text.draw(
            f,
            chip,
            r.x + r.w - cw - pad,
            r.y + (r.h - 12) / 2,
            11.0,
            INK_FAINT,
            a,
        );
    }

    fn draw_presence(&mut self, f: &mut Frame) {
        let (a, _) = self.enter(Element::Presence);
        if a <= 0.0 {
            return;
        }
        let thinking = Instant::now() < self.thinking_until;
        let period = if thinking { THINK_SECS } else { BREATH_SECS };
        let t = self.start.elapsed().as_secs_f32();
        let breath = 0.5 - 0.5 * (t * std::f32::consts::TAU / period).cos();

        let (cx, cy) = self.presence_c;
        let base = 7.0;
        let r = base * (0.86 + 0.22 * breath);
        f.glow(cx, cy, r, 24.0, PRESENCE, (0.10 + 0.14 * breath) * a);
        f.disc(cx, cy, r, PRESENCE, a);
    }
}
