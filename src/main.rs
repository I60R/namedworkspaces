use swayipc::{NodeType, NodeLayout};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sway_rx = swayipc::Connection::new()?;
    let mut sway_tx = swayipc::Connection::new()?;

    use swayipc::EventType::*;
    let events = sway_rx.subscribe([Window, Binding, Workspace])?;

    for e in events {
        match e {
            Ok(swayipc::Event::Workspace(e)) => {
                use swayipc::{WorkspaceEvent, WorkspaceChange::*};
                if let WorkspaceEvent { change: Init, .. } = *e {
                    if let Some(ws) = e.current {
                        set_new_workspace_name(ws)?;
                    }
                }
            },
            Ok(swayipc::Event::Window(e)) => {
                use swayipc::{WindowEvent, WindowChange::*};
                if let WindowEvent { change: Focus | Move | Floating, .. } = *e {
                    set_workspace_name(&mut sway_tx, e.container)?;
                }
            },
            Ok(swayipc::Event::Binding(_)) => {
                let focused = find_focused(sway_tx.get_tree()?);
                set_workspace_name(&mut sway_tx, focused)?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn set_new_workspace_name(ws: swayipc::Node) -> Result<(), Box<dyn std::error::Error>> {
    let ws_num = ws.num.expect("Workspaces should be numbered");

    let style = "color='lightgreen' baseline_shift='superscript' font_size='10pt'";
    let ws_new_name = format!("{ws_num}<span {style}>＋</span>");

    let mut sway = swayipc::Connection::new()?;

    sway.run_command(format!("rename workspace to {ws_new_name}"))?;

    Ok(())
}

fn set_workspace_name(
    sway: &mut swayipc::Connection,
    win: swayipc::Node,
) -> Result<(), Box<dyn std::error::Error>> {
    let outputs = sway.get_tree()?;

    let ws = find_workspace(outputs.clone(), &win);
    let parent = find_parent(outputs, &win);
    let siblings = parent.nodes.len();

    let win_name = win.app_id.clone()
        .or_else(|| win.window_properties.clone().and_then(|p| p.class.or(p.instance).clone()))
        .unwrap_or_default();

    let mut layout_icons = String::new();
    let mut new_workspace = false;

    let layout_icon = if win.node_type == NodeType::FloatingCon {
        if siblings == 0 { "▪" } else { "▣" }
    } else {
        if siblings == 1 {
            "■"
        } else {
            if parent.layout == NodeLayout::SplitH {
                if siblings == 2 {
                    if parent.nodes[0].id == win.id { "◧" } else { "◨" }
                } else {
                    for x in 0..siblings {
                        layout_icons.push_str(if parent.nodes[x].id == win.id { "▮" } else { "▯" });
                    }
                    &layout_icons
                }
            } else if parent.layout == NodeLayout::SplitV {
                if siblings == 2 {
                    if parent.nodes[0].id == win.id { "⬒" } else { "⬓" }
                } else {
                    "▤"
                }
            } else {
                new_workspace = true;
                ""
            }
        }
    };

    let layout_icon_style = "font_size='16pt' color='lightgreen'";
    let layout_icon = format!("<span {layout_icon_style}>{layout_icon}</span>");

    let ws_old_name = ws.name.expect("Unnamed workspace");
    let ws_num = ws.num.expect("Unnumbered workspace");

    let ws_icon_style = "baseline_shift='superscript' font_size='12pt' color='lightgreen'";
    let ws_icon = assign_icon(&win_name);

    let ws_name_style = "color='orange' baseline_shift='2pt'";
    let ws_name = if !new_workspace {
        format!(" {layout_icon} <span {ws_name_style}> {win_name} </span>")
    } else {
        String::from(" ")
    };

    let ws_new_name = format!("{ws_num}<span {ws_icon_style}>{ws_icon}</span>{ws_name}");

    sway.run_command(format!("rename workspace {ws_old_name} to {ws_new_name}"))?;

    Ok(())
}


fn find_focused(tree: swayipc::Node) -> swayipc::Node {
    let mut stack = Vec::with_capacity(tree.nodes.len() + tree.floating_nodes.len());
    stack.push(tree);

    while let Some(n) = stack.pop() {
        if n.focused {
            return n
        }
        stack.extend(n.nodes);
        stack.extend(n.floating_nodes);
    }

    unreachable!("cannot find focused window in")
}

fn find_parent(tree: swayipc::Node, win: &swayipc::Node) -> swayipc::Node {
    let mut stack = Vec::with_capacity(tree.nodes.len() + tree.floating_nodes.len());
    stack.push(tree);

    while let Some(n) = stack.pop() {
        if n.nodes.iter().any(|n| n.id == win.id) {
            return n
        }
        if n.floating_nodes.iter().any(|n| n.id == win.id) {
            return n
        }
        stack.extend(n.nodes);
        stack.extend(n.floating_nodes);
    }

    unreachable!("cannot find parent for {}", win.id)
}

fn find_workspace(outputs: swayipc::Node, win: &swayipc::Node) -> swayipc::Node {
    let workspaces: Vec<_> = outputs.nodes.into_iter()
        .flat_map(|outputs| outputs.nodes)
        .filter(|workspace| workspace.name.as_deref() != Some("__i3_scratch"))
        .collect();

    for ws in workspaces {
        let ws_clone = ws.clone();
        if ws.find(|n| n.id == win.id).is_some() {
            return ws_clone
        }
    }

    unreachable!("cannot find active workspace")
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
        "" => "＋",
        _ => "?",
    }
}

