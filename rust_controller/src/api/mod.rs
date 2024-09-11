use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;

use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::log::LogLevel;
use rocket::tokio::sync::broadcast::channel;
use rocket::{Build, Rocket, Route};
use rocket_dyn_templates::Template;
use rocket_okapi::openapi_get_routes;
use rocket_okapi::rapidoc::{make_rapidoc, GeneralConfig, RapiDocConfig};
use rocket_okapi::settings::UrlObject;
use rocket_okapi::swagger_ui::{make_swagger_ui, SwaggerUIConfig};
use tokio::sync::{Mutex, MutexGuard};

pub use guard::Error;
use guard::*;
use mutation::*;
pub use websocket::channels::{DivisionUpdate, HoleUpdate, PlayerManagerUpdate};

pub use crate::api::websocket::channels::GeneralChannel;
use crate::api::websocket::htmx::division_updater;
use crate::api::websocket::HoleFinishedAlert;
use crate::controller::coordinator::FlipUpVMixCoordinator;
use crate::util;

mod coordinator_wrapper;
mod guard;
mod mutation;
mod query;
mod update_loop;
mod vmix_calls;
mod webpage_responses;
mod websocket;

use super::controller::coordinator::leaderboard_cycle;

#[derive(Debug, Clone)]
struct Coordinator(Arc<Mutex<FlipUpVMixCoordinator>>);

impl FlipUpVMixCoordinator {
    pub async fn into_coordinator(
        self,
        hole_finished_alert: GeneralChannel<HoleFinishedAlert>,
    ) -> Coordinator {
        let next_group = self.next_group.clone();
        let coordinator = Arc::new(Mutex::new(self));
        let s = Coordinator(coordinator.clone());
        let leaderboard_cycle =
            leaderboard_cycle::start_leaderboard_cycle(coordinator.clone()).await;
        tokio::spawn(async move {
            update_loop::update_loop(
                coordinator,
                leaderboard_cycle,
                hole_finished_alert,
                next_group,
            )
            .await;
        });

        s
    }
}

impl Coordinator {
    async fn lock(&self) -> MutexGuard<FlipUpVMixCoordinator> {
        self.0.lock().await
    }
}

fn get_normal_routes() -> Vec<Route> {
    use mutation::*;
    use query::*;
    use vmix_calls::*;
    use webpage_responses::{index, okapi_add_operation_for_index_};
    openapi_get_routes![
        current_hole,
        amount_of_rounds,
        current_round,
        get_players,
        get_divisions,
        set_focus,
        load,
        set_group,
        play_animation,
        get_groups,
        index,
        update_leaderboard,
        update_division,
        increase_score,
        focused_players,
        focused_player,
        revert_score,
        increase_throw,
        revert_throw,
        play_ob_animation,
        set_hole_info,
        update_other_leaderboard,
        next_queue,
        add_to_queue,
        focused_card,
        set_group_to_focused_player,
        next_10_lb,
        reset_lb_pos,
        rewind_lb_pos,
        next_featured_hole_card,
        update_featured_hole_group,
        rewind_featured_hole_card,
        set_leaderboard_round,
        set_hole
    ]
}

fn get_websocket_routes() -> Vec<Route> {
    use websocket::*;
    routes![
        selection_watcher,
        hole_watcher,
        division_updater,
        leaderboard_round_watcher,
        hole_finished_alert
    ]
}

fn get_websocket_htmx_routes() -> Vec<Route> {
    use websocket::htmx::*;
    routes![selection_updater, focused_player_changer, leaderboard_round]
}

fn get_webpage_routes() -> Vec<Route> {
    use webpage_responses::*;

    openapi_get_routes![focused_players, set_group, load]
}

pub fn launch() -> Rocket<Build> {
    let (group_selection_sender, _) = channel::<websocket::PlayerManagerUpdate>(1024);
    let group_selection_sender = GeneralChannel::from(group_selection_sender);
    let (hole_update_sender, _) = channel::<HoleUpdate>(1024);
    let hole_update_sender = GeneralChannel::from(hole_update_sender);
    let division_sender = GeneralChannel::from(channel::<websocket::DivisionUpdate>(1024).0);
    let round_sender = GeneralChannel::from(channel::<websocket::LeaderboardRoundUpdate>(1024).0);
    let hole_finished_alert = GeneralChannel::from(channel::<websocket::HoleFinishedAlert>(1024).0);

    let conf = {
        #[cfg(windows)]
        let ip = IpAddr::V4("10.170.121.242".parse().unwrap());
        #[cfg(not(windows))]
        let ip = IpAddr::V4("10.180.121.3".parse().unwrap());
        rocket::Config {
            address: ip,
            cli_colors: true,
            log_level: LogLevel::Normal,
            ..Default::default()
        }
    };

    rocket::build()
        .configure(conf)
        .manage(CoordinatorLoader(Mutex::new(None)))
        .manage(group_selection_sender)
        .manage(hole_update_sender)
        .manage(division_sender)
        .manage(round_sender)
        .manage(hole_finished_alert)
        .mount("/", get_normal_routes())
        .mount("/htmx/", get_webpage_routes())
        .mount("/ws", get_websocket_routes())
        .mount("/ws/htmx/", get_websocket_htmx_routes())
        .mount("/static", {
            if cfg!(target_os = "windows") {
                FileServer::from("C:\\livegrafik-flipup\\_conf\\static")
            } else {
                FileServer::from("static")
            }
        })
        .attach(Template::fairing())
        .register("/", catchers![make_coordinator,])
        .mount(
            "/api/swagger",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/api",
            make_rapidoc(&RapiDocConfig {
                general: GeneralConfig {
                    spec_urls: vec![UrlObject::new("General", "../openapi.json")],
                    ..Default::default()
                },
                ..Default::default()
            }),
        )
        .attach(AdHoc::on_shutdown("Shutdown Printer", |_| {
            Box::pin(async move {
                util::delete_files_in_directory(Path::new("images")).unwrap();
            })
        }))
}
