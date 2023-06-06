#![windows_subsystem = "windows"]
#![feature(drain_filter)]
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

use directories::ProjectDirs;

use iced::theme::{self, Theme};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, Image};
use iced::{Element, Length, Sandbox, Settings};

use itertools::Itertools;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use tft::tft_data::*;

static DIR: OnceLock<ProjectDirs> = OnceLock::new();
static CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Default)]
enum Screen {
    #[default]
    CharacterBuilder,
    ItemDeterminer,
}

#[derive(Debug, Clone)]
enum Message {
    ClickedChampion(String),
    ClickedComponentAdd(Item),
    ClickedComponentSub(Item),
    ClickedItem(Item),
    ClickedItemRemove(Item),
    ClearItems(String),
    ClickedSave,
    ChangeScreen,
    ChangeSortMethod(SortChampMethod),
}

struct Model {
    screen: Screen,
    champs: Vec<ChampionState>,
    items: Vec<Item>,
    components: Vec<ComponentState>,
    focused_champion: Option<String>,
    curr_sort_method: SortChampMethod,
}

#[derive(Debug, Default, Deserialize, Clone, Serialize)]
struct ChampionState {
    champ: Champion,
    items: Vec<Item>,
}

#[derive(Debug, Default, Deserialize, Clone, Serialize)]
struct ComponentState {
    component: Item,
    count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum SortChampMethod {
    #[default]
    Alphabetical,
    Cost,
    Trait,
}

impl ToString for SortChampMethod {
    fn to_string(&self) -> String {
        match self {
            SortChampMethod::Alphabetical => String::from("Alphabetical"),
            SortChampMethod::Cost => String::from("By Cost"),
            SortChampMethod::Trait => String::from("By Trait"),
        }
    }
}

impl Sandbox for Model {
    type Message = Message;
    // type Theme = Theme;
    // type Executor = executor::Default;
    // type Flags = ();

    fn new() -> Self {
        let f = ureq::get("https://raw.communitydragon.org/latest/cdragon/tft/en_us.json")
            .call()
            .unwrap()
            .into_string()
            .unwrap();
        let json: Value = serde_json::from_str(&f).unwrap();
        let obj = json.as_object().unwrap();

        // access tft set 8 stage 2 champions
        let mut champs: Vec<Champion> = serde_json::from_value(
            obj.get("setData")
                .unwrap()
                .get(18) // tft set 8 stage 2
                .unwrap()
                .get("champions")
                .unwrap()
                .clone(),
        )
        .unwrap();

        // remove champions that have no traits (eggs, creeps, etc.)
        champs.drain_filter(|champ| champ.traits.is_empty());

        // get items
        let mut items: Vec<Item> =
            serde_json::from_value(obj.get("items").unwrap().clone()).unwrap();
        // only keep items composed of other items (standard completed items)
        items.drain_filter(|item| item.composition.is_empty());
        // remove items that are exclusive to particular sets
        items.drain_filter(|item| {
            item.api_name.contains('5') // remove set 5 exclusives
                || item.api_name.contains('6') // remove set 6 exclusives
                || item.api_name.contains('7') // remove set 7 exclusives
                || item.name.contains("tft_item_name") // remove special tft items
                || item.composition.iter().any(|component| component.contains("Tutorial"))
        });
        let mut set = HashSet::new();
        for item in items.iter() {
            for component in item.composition.iter() {
                set.insert(component.clone());
            }
        }
        let components: Vec<ComponentState> = set
            .into_iter()
            .map(|component_api_name| {
                let all_items: Vec<Item> =
                    serde_json::from_value(obj.get("items").unwrap().clone()).unwrap();
                ComponentState {
                    component: all_items
                        .into_iter()
                        .find(|item| item.api_name == component_api_name)
                        .unwrap(),
                    count: 0,
                }
            })
            .collect();
        // println!("{components:#?}");

        let champ_state = champs
            .into_iter()
            .map(|champ| {
                let data_dir = DATA_DIR.get().unwrap();
                let path = "champ_info.json";
                if let Ok(s) = fs::read_to_string(data_dir.join(path)) {
                    let x: Vec<ChampionState> = serde_json::from_str(&s).unwrap();
                    let y: Vec<ChampionState> = x
                        .into_iter()
                        .filter(|champ_state| champ_state.champ.name == champ.name)
                        .collect();
                    y[0].clone()
                } else {
                    ChampionState {
                        champ,
                        items: vec![],
                    }
                }
            })
            .collect();
        // let x = pane_grid::State::
        // let (items, _) = pane_grid::State::new(Item::default());
        Model {
            screen: Screen::default(),
            champs: champ_state,
            items,
            components,
            focused_champion: None,
            curr_sort_method: SortChampMethod::default(),
        }
    }

    fn title(&self) -> String {
        String::from("TFT App")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ClickedChampion(name) => {
                println!("new focused champion is {}", name);
                self.focused_champion = Some(name);
            }
            Message::ClickedItem(item) => {
                if let Some(ref champ) = self.focused_champion {
                    println!("{} got added to {}", item, champ);
                    let champ = self
                        .champs
                        .iter_mut()
                        .find(|champ_state| &champ_state.champ.name == champ)
                        .unwrap();
                    champ.items.push(item);
                }
            }
            Message::ClickedItemRemove(item) => {
                if let Some(ref champ) = self.focused_champion {
                    println!("{} got added to {}", item, champ);
                    let champ = self
                        .champs
                        .iter_mut()
                        .find(|champ_state| &champ_state.champ.name == champ)
                        .unwrap();
                    if let Some(index) = champ.items.iter().position(|x| x.name == item.name) {
                        champ.items.remove(index);
                    }
                }
            }
            Message::ClickedSave => {
                let data_dir = DATA_DIR.get().unwrap();
                let s = serde_json::to_string(&self.champs).unwrap();
                fs::write(data_dir.join("champ_info.json"), s).unwrap();
            }
            Message::ChangeScreen => {
                self.screen = match self.screen {
                    Screen::CharacterBuilder => Screen::ItemDeterminer,
                    Screen::ItemDeterminer => Screen::CharacterBuilder,
                };
            }
            Message::ClickedComponentAdd(component) => {
                let component: &mut ComponentState = self
                    .components
                    .iter_mut()
                    .find(|component_state| component_state.component.name == component.name)
                    .unwrap();
                component.count += 1;
            }
            Message::ClickedComponentSub(component) => {
                let component: &mut ComponentState = self
                    .components
                    .iter_mut()
                    .find(|component_state| component_state.component.name == component.name)
                    .unwrap();
                component.count = component.count.saturating_sub(1);
            }
            Message::ClearItems(champ) => {
                let champ = self
                    .champs
                    .iter_mut()
                    .find(|champ_state| champ_state.champ.name == champ)
                    .unwrap();
                champ.items.clear();
            }
            Message::ChangeSortMethod(method) => {
                self.curr_sort_method = method;
            }
        }
    }

    fn view(&self) -> Element<Message> {
        match self.screen {
            Screen::CharacterBuilder => {
                let mut champs_clone = self.champs.clone();
                champs_clone.sort_by(|a, b| match self.curr_sort_method {
                    SortChampMethod::Alphabetical => (a.champ.name).cmp(&b.champ.name),
                    SortChampMethod::Cost => (a.champ.cost).cmp(&b.champ.cost),
                    SortChampMethod::Trait => (a.champ.traits).cmp(&b.champ.traits),
                });
                let chunks = champs_clone.into_iter().chunks(3);
                let mut rows = vec![];
                // let mut rows = column!(row!(Image::new(image::Handle::default())));
                for chunk in &chunks {
                    rows.push(row(chunk
                        .into_iter()
                        .map(|a| {
                            column!(
                                Image::new(a.champ.square_icon.handle.clone()),
                                button(text(a.champ.name.clone()))
                                    .on_press(Message::ClickedChampion(a.champ.name.clone())),
                                button(text("Clear"))
                                    .on_press(Message::ClearItems(a.champ.name))
                                    .style(iced::theme::Button::Destructive)
                            )
                            .into()
                        })
                        .collect::<Vec<_>>()));
                }
                let champion_col = rows.into_iter().fold(
                    column!(pick_list(
                        vec![
                            SortChampMethod::Alphabetical,
                            SortChampMethod::Cost,
                            SortChampMethod::Trait
                        ],
                        Some(self.curr_sort_method),
                        |method| { Message::ChangeSortMethod(method) }
                    )),
                    |col, row| col.push(row),
                );

                let item_chunks = self.items.clone().into_iter().chunks(3);
                let mut item_rows = vec![];
                for item_chunk in &item_chunks {
                    item_rows.push(row(item_chunk
                        .into_iter()
                        .map(|a| {
                            column!(
                                Image::new(a.icon.handle.clone()),
                                row!(
                                    button(text(a.name.clone()))
                                        .on_press(Message::ClickedItem(a.clone())),
                                    button(text("-"))
                                        .on_press(Message::ClickedItemRemove(a))
                                        .style(iced::theme::Button::Destructive)
                                )
                            )
                            .into()
                        })
                        .collect::<Vec<_>>()))
                }

                let item_col = item_rows
                    .into_iter()
                    .fold(column!(), |col, row| col.push(row));

                container(row!(
                    scrollable(champion_col),
                    scrollable(item_col),
                    column!(
                        text(match self.focused_champion.clone() {
                            Some(champ) => {
                                let champ = self
                                    .champs
                                    .iter()
                                    .find(|champ_state| champ_state.champ.name == champ)
                                    .unwrap();
                                format!("{}: {}", champ.champ, ItemsDisplay(champ.items.clone()))
                            }
                            None => String::from("No champion selected"),
                        }),
                        button(text("Save")).on_press(Message::ClickedSave),
                        button(text("Go to Item Determiner")).on_press(Message::ChangeScreen),
                    )
                ))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            Screen::ItemDeterminer => {
                let item_chunks = self.components.clone().into_iter().chunks(3);
                let mut item_rows = vec![];
                for item_chunk in &item_chunks {
                    item_rows.push(row(item_chunk
                        .into_iter()
                        .map(|a| {
                            column!(
                                Image::new(a.component.icon.handle.clone()),
                                row!(
                                    text(a.component.name.clone()),
                                    button(text("+")).on_press(Message::ClickedComponentAdd(
                                        a.component.clone()
                                    )),
                                    text(a.count),
                                    button(text("-"))
                                        .on_press(Message::ClickedComponentSub(a.component))
                                        .style(iced::theme::Button::Destructive)
                                )
                            )
                            .into()
                        })
                        .collect::<Vec<_>>()))
                }

                let item_col = item_rows
                    .into_iter()
                    .fold(column!(), |col, row| col.push(row));

                // now show the champions that like these items
                let mut sorted_champs = self.champs.clone();
                sorted_champs.sort_by(|a, b| {
                    let a_items: Vec<String> = a
                        .items
                        .iter()
                        .flat_map(|item| item.composition.clone())
                        .collect();
                    let b_items: Vec<String> = b
                        .items
                        .iter()
                        .flat_map(|item| item.composition.clone())
                        .collect();
                    let a_map = a_items.iter().fold(HashMap::new(), |mut acc, c| {
                        *acc.entry(c).or_insert(0usize) += 1;
                        acc
                    });
                    let b_map = b_items.iter().fold(HashMap::new(), |mut acc, c| {
                        *acc.entry(c).or_insert(0usize) += 1;
                        acc
                    });

                    let comp_map = self.components.iter().fold(HashMap::new(), |mut acc, c| {
                        acc.insert(&c.component.api_name, c.count);
                        acc
                    });

                    let mut a_total = 0;
                    let mut b_total = 0;

                    for (comp_name, comp_count) in comp_map.iter() {
                        for (a_name, a_count) in a_map.clone() {
                            if comp_name == &a_name {
                                a_total += if *comp_count > a_count {
                                    a_count
                                } else {
                                    *comp_count
                                };
                            }
                        }
                    }
                    for (comp_name, comp_count) in comp_map {
                        for (b_name, b_count) in b_map.clone() {
                            if comp_name == b_name {
                                b_total += if comp_count > b_count {
                                    b_count
                                } else {
                                    comp_count
                                };
                            }
                        }
                    }
                    b_total.cmp(&a_total)
                });

                let chunks = sorted_champs.into_iter().chunks(3);
                let mut rows = vec![];
                // let mut rows = column!(row!(Image::new(image::Handle::default())));
                for chunk in &chunks {
                    rows.push(row(chunk
                        .into_iter()
                        .map(|a| {
                            column!(
                                Image::new(a.champ.square_icon.handle.clone()),
                                text(a.champ.name)
                            )
                            .into()
                        })
                        .collect::<Vec<_>>()));
                }
                let champion_col = rows.into_iter().fold(column!(), |col, row| col.push(row));

                container(row!(
                    item_col,
                    scrollable(champion_col),
                    button(text("Go to Character Builder")).on_press(Message::ChangeScreen)
                ))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
        }
    }

    fn theme(&self) -> Theme {
        Theme::default()
    }

    fn style(&self) -> theme::Application {
        theme::Application::default()
    }

    fn scale_factor(&self) -> f64 {
        1.0
    }

    fn run(settings: Settings<()>) -> Result<(), iced::Error>
    where
        Self: 'static + Sized,
    {
        <Self as iced::Application>::run(settings)
    }
}

fn main() {
    // initialize logger
    env_logger::builder().format_timestamp(None).init();

    // set up directories
    let dir = DIR.get_or_init(|| ProjectDirs::from("", "Sharif Haason", "TFT_Notebook").unwrap());
    CACHE_DIR.get_or_init(|| {
        fs::create_dir_all(dir.cache_dir()).unwrap();
        dir.cache_dir().to_path_buf()
    });
    DATA_DIR.get_or_init(|| {
        fs::create_dir_all(dir.data_dir()).unwrap();
        dir.data_dir().to_path_buf()
    });

    ureq::get("https://raw.communitydragon.org/latest/cdragon/tft/en_us.json")
        .call()
        .unwrap();

    Model::run(Settings {
        antialiasing: true,
        window: iced::window::Settings {
            position: iced::window::Position::Centered,
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
    .unwrap()

    // ok
    // now we have to read a file that gives bis
    // maybe use toml

    // println!("{items:#?}");
}
