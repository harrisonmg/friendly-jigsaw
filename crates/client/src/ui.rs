use std::{
    iter::{Cycle, Peekable},
    slice::Iter,
    time::Duration,
};

use bevy::{prelude::*, time::common_conditions::on_timer};
use game::Puzzle;
use gloo_file::{Blob, ObjectUrl};
use wasm_bindgen::JsCast;
use web_sys::HtmlAnchorElement;

use crate::{
    colors::{DARK, LIGHTER},
    states::AppState,
    util::despawn,
};

pub struct UiPlugin;

const DEFAULT_LOADING_TEXT: &str = "Connecting to server";

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // setup
        app.add_systems(Startup, (startup, start_loading))
            .insert_resource(LoadingMessage(String::from(DEFAULT_LOADING_TEXT)))
            .add_systems(OnEnter(AppState::Downloading), start_loading);

        // loading
        app.add_systems(
            Update,
            loading_display.run_if(not(in_state(AppState::Playing))),
        )
        .add_systems(
            Update,
            loading_animation
                .run_if(not(in_state(AppState::Playing)))
                .run_if(on_timer(Duration::from_millis(150))),
        );

        // playing
        app.add_systems(OnEnter(AppState::Playing), (enter_playing, stop_loading))
            .add_systems(OnExit(AppState::Playing), (exit_playing, start_loading))
            .add_systems(Update, hover_help.run_if(in_state(AppState::Playing)))
            .add_systems(
                Update,
                hover_image_download.run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Resource)]
struct UiFont(Handle<Font>);

#[derive(Component)]
struct LoadingNode;

#[derive(Component)]
struct LoadingText;

#[derive(Resource)]
struct LoadingTextCycle<'a>(Peekable<Cycle<Iter<'a, &'a str>>>);

#[derive(Resource, Default)]
pub struct LoadingMessage(pub String);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.insert_resource(UiFont(font_handle.clone()));

    let cycle = [" . . .", " · . .", " . · .", " . . ·", " . . ."]
        .iter()
        .cycle()
        .peekable();
    commands.insert_resource(LoadingTextCycle(cycle));
}

fn start_loading(mut commands: Commands, font: Res<UiFont>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .insert(LoadingNode)
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: DARK.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            DEFAULT_LOADING_TEXT,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 25.0,
                                color: LIGHTER,
                            },
                        ))
                        .insert(LoadingText);
                });
        });
}

fn loading_animation(mut cycle: ResMut<LoadingTextCycle<'static>>) {
    let _ = *cycle.0.next().unwrap();
}

fn loading_display(
    mut text_query: Query<&mut Text, With<LoadingText>>,
    mut cycle: ResMut<LoadingTextCycle<'static>>,
    loading_msg: Res<LoadingMessage>,
) {
    let mut text = text_query.get_single_mut().unwrap();
    let msg = format!("{}{}", loading_msg.0, *cycle.0.peek().unwrap());
    text.sections[0].value = msg;
}

fn stop_loading(mut commands: Commands, loading_msg_query: Query<Entity, With<LoadingNode>>) {
    despawn(loading_msg_query, commands);
}

#[derive(Component)]
struct HelpNode;

#[derive(Component)]
struct HelpButton;

#[derive(Component)]
struct HelpText;

const HELP_SYMBOL: &str = "?";
const HELP_TEXT: &str = "• Left click and drag to move a piece\n\
                        • Right or middle click and drag to pan\n\
                        • Scroll to zoom\n\
                        • Press space to center the camera\n\n\
                        Made by Harrison Gieraltowski - harrisonmg.net";

#[derive(Component)]
struct ImageDownloadNode;

#[derive(Component)]
struct ImageDownloadButton;

#[derive(Component)]
struct ImageDownloadText;

const IMAGE_DOWNLOAD_SYMBOL: &str = "↓";
const IMAGE_DOWNLOAD_TEXT: &str = "Click to download the full puzzle image";

const BUTTON_SIZE: Val = Val::Px(30.0);

fn enter_playing(mut commands: Commands, font: Res<UiFont>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::End,
                justify_content: JustifyContent::Start,
                ..default()
            },
            ..default()
        })
        .insert(HelpNode)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: BUTTON_SIZE,
                        height: BUTTON_SIZE,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: DARK.into(),
                    ..default()
                })
                .insert(HelpButton)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            HELP_SYMBOL,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 25.0,
                                color: LIGHTER,
                            },
                        ))
                        .insert(HelpText);
                });
        });

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::End,
                justify_content: JustifyContent::End,
                ..default()
            },
            ..default()
        })
        .insert(ImageDownloadNode)
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: BUTTON_SIZE,
                        height: BUTTON_SIZE,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(10.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    background_color: DARK.into(),
                    ..default()
                })
                .insert(ImageDownloadButton)
                .with_children(|parent| {
                    parent
                        .spawn(TextBundle::from_section(
                            IMAGE_DOWNLOAD_SYMBOL,
                            TextStyle {
                                font: font.0.clone(),
                                font_size: 25.0,
                                color: LIGHTER,
                            },
                        ))
                        .insert(ImageDownloadText);
                });
        });
}

fn exit_playing(
    mut commands: Commands,
    help_node_query: Query<Entity, With<HelpNode>>,
    image_download_node_query: Query<Entity, With<ImageDownloadNode>>,
) {
    despawn(help_node_query, commands);
    despawn(image_download_node_query, commands);
}

fn hover_help(
    mut interaction_query: Query<(&Interaction, &mut Style), With<HelpButton>>,
    mut text_query: Query<&mut Text, With<HelpText>>,
) {
    let mut text = text_query.get_single_mut().unwrap();
    for (interaction, mut style) in &mut interaction_query {
        match interaction {
            Interaction::None => {
                text.sections[0].value = String::from(HELP_SYMBOL);
                style.width = BUTTON_SIZE;
                style.height = BUTTON_SIZE;
            }
            Interaction::Hovered => {
                text.sections[0].value = String::from(HELP_TEXT);
                style.width = Val::Auto;
                style.height = Val::Auto;
            }
            _ => (),
        }
    }
}

fn hover_image_download(
    mut clicked: Local<bool>,
    mut interaction_query: Query<(&Interaction, &mut Style), With<ImageDownloadButton>>,
    mut text_query: Query<&mut Text, With<ImageDownloadText>>,
    puzzle: Res<Puzzle>,
) {
    let mut text = text_query.get_single_mut().unwrap();
    for (interaction, mut style) in &mut interaction_query {
        match interaction {
            Interaction::None => {
                text.sections[0].value = String::from(IMAGE_DOWNLOAD_SYMBOL);
                style.width = BUTTON_SIZE;
                style.height = BUTTON_SIZE;
                *clicked = false;
            }
            Interaction::Hovered => {
                text.sections[0].value = String::from(IMAGE_DOWNLOAD_TEXT);
                style.width = Val::Auto;
                style.height = Val::Auto;
                *clicked = false;
            }

            Interaction::Clicked => {
                if !*clicked {
                    *clicked = true;

                    let bytes = puzzle.raw_image();
                    let blob = Blob::new(bytes.as_ref());
                    let object_url = ObjectUrl::from(blob);

                    let window = web_sys::window().unwrap();
                    let document = window.document().unwrap();

                    let link = document
                        .create_element("a")
                        .unwrap()
                        .dyn_into::<HtmlAnchorElement>()
                        .unwrap();
                    link.style().set_property("display", "none").unwrap();
                    link.set_href(object_url.as_ref());
                    link.set_download("cheater.png");

                    let body = document.body().unwrap();
                    body.append_child(&link).unwrap();
                    link.click();
                }
            }
        }
    }
}
