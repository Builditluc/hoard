use crate::command::hoard_command::HoardCommand;
use crate::command::trove::CommandTrove;
use crate::config::HoardConfig;
use crate::gui::help::{draw as draw_help, key_handler as key_handler_help};
use crate::gui::list_search::controls::key_handler as key_handler_list_search;
use crate::gui::list_search::render::draw as draw_list_search;
use crate::gui::parameter_input::controls::key_handler as key_handler_parameter_input;
use crate::gui::parameter_input::render::draw as draw_parameter_input;
use crossterm::event::{poll, read, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use eyre::Result;
use std::io::stdout;
use std::time::Duration;
use tui::{backend::CrosstermBackend, widgets::ListState, Terminal};

pub struct State {
    pub input: String,
    pub commands: Vec<HoardCommand>,
    pub command_list_state: ListState,
    pub namespace_tab_state: ListState,
    pub should_exit: bool,
    pub draw_state: DrawState,
    pub parameter_token: String,
    pub selected_command: Option<HoardCommand>,
    pub provided_parameter_count: u16,
}

#[derive(Debug, Eq, PartialEq)]
pub enum DrawState {
    Search,
    ParameterInput,
    Help,
}

#[allow(clippy::too_many_lines)]
pub fn run(trove: &mut CommandTrove, config: &HoardConfig) -> Result<Option<HoardCommand>> {
    let mut app_state = State {
        input: "".to_string(),
        commands: trove.commands.clone(),
        command_list_state: ListState::default(),
        namespace_tab_state: ListState::default(),
        should_exit: false,
        draw_state: DrawState::Search,
        parameter_token: config.parameter_token.as_ref().unwrap().clone(),
        selected_command: None,
        provided_parameter_count: 0,
    };

    app_state.command_list_state.select(Some(0));
    app_state.namespace_tab_state.select(Some(0));

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    //let menu_titles = vec!["List", "Search", "Add", "Delete", "Quit"];
    let mut namespace_tabs: Vec<&str> = trove.namespaces();
    namespace_tabs.insert(0, "All");
    loop {
        // Draw GUI
        match app_state.draw_state {
            DrawState::Search => {
                draw_list_search(&mut app_state, config, &mut namespace_tabs, &mut terminal)?;
            }
            DrawState::ParameterInput => {
                draw_parameter_input(&mut app_state, config, &mut terminal)?;
            }
            DrawState::Help => {
                draw_help(config, &mut terminal)?;
            }
        }

        if poll(Duration::from_secs(0))? {
            if let Event::Key(input) = read()? {
                let command = match app_state.draw_state {
                    DrawState::Search => key_handler_list_search(
                        input,
                        &mut app_state,
                        &trove.commands,
                        &namespace_tabs,
                    ),
                    DrawState::ParameterInput => key_handler_parameter_input(input, &mut app_state),
                    DrawState::Help => key_handler_help(input, &mut app_state),
                };

                if let Some(output) = command {
                    disable_raw_mode()?;
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                    terminal.show_cursor()?;
                    return Ok(Some(output));
                }

                if app_state.should_exit {
                    disable_raw_mode()?;
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                    terminal.show_cursor()?;
                    return Ok(None);
                }
            }
        }
    }
}
