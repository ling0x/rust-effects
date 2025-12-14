use std::{cell::RefCell, io, rc::Rc, time::Duration};

use ratatui::{
    layout::{Alignment, Offset, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Paragraph},
    Frame, Terminal,
};

use ratzilla::{
    event::{KeyCode, KeyEvent},
    DomBackend, WebRenderer,
};

use tachyonfx::{fx, fx::EvolveSymbolSet, pattern, CellFilter, EffectManager, Interpolation};

fn main() -> io::Result<()> {
    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    let state = Rc::new(App::default());

    let event_state = Rc::clone(&state);
    terminal.on_key_event(move |key_event| {
        event_state.handle_events(key_event);
    });

    let render_state = Rc::clone(&state);
    terminal.draw_web(move |frame| {
        render_state.render(frame);
    });

    Ok(())
}

#[derive(Default)]
struct App {
    counter: RefCell<u8>,
    effects: RefCell<EffectManager<()>>,
}

impl App {
    fn render(&self, frame: &mut Frame) {
        let counter = self.counter.borrow();
        let block = Block::bordered()
            .title("rust-effects")
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded);

        // Static header text
        let header_text = format!(
            r#"This is a Ratzilla template.
            Press left and right to increment and decrement the counter respectively.
            Press 'f' to trigger fire effect.
            Counter: {counter}
            "#
        );

        // Code snippet that will receive the fire effect
        let code_snippet = r#"use ratatui::style::Stylize;
        use ratatui::widgets::{Block, Paragraph};

        fn main() -> Result<(), Box<dyn std::error::Error>> {
            let mut terminal = ratatui::init();
            terminal.draw(|frame| {
                let block = Block::bordered().title("Welcome");
                let greeting = Paragraph::new("Hello, Ratatui! 🐭")
                    .centered()
                    .on_cyan()
                    .block(block);
                frame.render_widget(greeting, frame.area());
            })?;
            std::thread::sleep(std::time::Duration::from_secs(5));
            ratatui::restore();
            Ok(())
        }
        "#;

        // Combine texts
        let full_text = format!("{}{}", header_text, code_snippet);

        let paragraph = Paragraph::new(full_text)
            .block(block)
            .fg(Color::White)
            .bg(Color::Black)
            .centered();

        frame.render_widget(paragraph, frame.area());

        let area = frame.area();

        let mut effects = self.effects.borrow_mut();
        effects.process_effects(Duration::from_millis(16).into(), frame.buffer_mut(), area);
    }

    fn handle_events(&self, key_event: KeyEvent) {
        let mut counter = self.counter.borrow_mut();
        match key_event.code {
            KeyCode::Left => *counter = counter.saturating_sub(1),
            KeyCode::Right => *counter = counter.saturating_add(1),
            KeyCode::Char('f') => {
                drop(counter);
                self.trigger_fire_effect();
            }
            _ => {}
        }
    }

    fn trigger_fire_effect(&self) {
        // Calculate the area where the code snippet starts
        // Adjust these values based on your layout:
        // - x: horizontal offset from left edge
        // - y: vertical offset (header takes ~5 lines)
        // - width: width of the code block
        // - height: height of the code block
        let code_area = Rect::new(12, 12, 80, 17);

        let screen_bg = Color::from_u32(0x1D2021);
        let content_bg = Color::from_u32(0x32302F);

        let style = Style::default().fg(content_bg).bg(screen_bg);

        let boot_timer = (300, Interpolation::CircIn);
        let timer = (900, Interpolation::QuadIn);

        // Phase 1: Startup - Radial pattern evolve effect (fire ignition)
        let startup = fx::evolve((EvolveSymbolSet::Shaded, style), boot_timer)
            .with_pattern(pattern::RadialPattern::with_transition((0.5, 0.5), 10.0))
            .with_area(code_area);

        // Phase 2: Main Fire - Reversed evolve_from with coalesce pattern
        let inner_fire_fx = fx::evolve_from((EvolveSymbolSet::Quadrants, style), timer)
            .with_pattern(pattern::CoalescePattern::new())
            .with_area(code_area)
            .reversed();

        // Translate the fire upward to simulate rising flames
        let fire =
            fx::translate(inner_fire_fx, Offset { x: 0, y: -22 }, timer).with_area(code_area);

        // Phase 3: Text Fade-In - Reveals text through the fire with coalesce pattern
        let fade_in_text = fx::fade_from(screen_bg, screen_bg, timer)
            .with_filter(CellFilter::Text)
            .with_area(code_area)
            .with_pattern(pattern::CoalescePattern::new());

        // Orchestrate all phases
        let fire_effect = fx::prolong_start(
            300,
            fx::sequence(&[
                startup,
                fx::parallel(&[fx::fade_from(screen_bg, screen_bg, 300), fire, fade_in_text]),
            ]),
        );

        let mut effects = self.effects.borrow_mut();
        effects.add_effect(fire_effect);
    }
}
