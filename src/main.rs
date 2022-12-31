
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection = swayipc::Connection::new()?;

    use swayipc::EventType::*;
    let events = connection.subscribe([Workspace, Window, Binding])?;

    for e in events {
        match e {
            Ok(swayipc::Event::Workspace(e)) => {
                use swayipc::{WorkspaceEvent, WorkspaceChange::*};
                if let WorkspaceEvent { change: Init | Focus | Move | Reload, .. } = *e {
                    if let Some(current) = e.current {
                        assign_generic_name(current)
                    }
                }
            },
            Ok(swayipc::Event::Window(e)) => {
                use swayipc::{WindowEvent, WindowChange::*};
                if let WindowEvent { change: New | Focus | Title | Move, .. } = *e {
                    assign_generic_name(e.container)
                }
            },
            _ => {}
        }
    }

    Ok(())
}

fn assign_generic_name(unwrap: swayipc::Node) {
    todo!()
}


fn assign_icon(app_id: &str) -> &str {
    match app_id {
        "firefox" => "",
        "neovide" => "",
        "Code" => "",
        "Chromium" => "",
        "gthumb" => "",
        "swappy" => "",
        "org.twosheds.iwgtk" => "直",
        "org.gnome.Weather" => "",
        "org.kde.krusader" => "",
        "albert" => "",
        "gnome_system_monitor" => "",
        _ => "?",
    }
}

