use swayipc::{NodeType, NodeLayout};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sway_rx = swayipc::Connection::new()?;

    use swayipc::EventType::*;
    let events = sway_rx.subscribe([Window, Binding, Workspace])?;

    for e in events {
        match e {
            Ok(swayipc::Event::Window(e)) => {
                use swayipc::{WindowEvent, WindowChange::*};
                if let WindowEvent { change: Focus | Move | Floating, .. } = *e {
                    let mut sway_tx = swayipc::Connection::new()?;
                    let tree = &sway_tx.get_tree()?;
                    set_workspace_name(&mut sway_tx, tree, &e.container)?;
                }
            },
            Ok(swayipc::Event::Binding(_)) => {
                let mut sway_tx = swayipc::Connection::new()?;
                let tree = sway_tx.get_tree()?;
                let focused = find_focused(&tree);
                set_workspace_name(&mut sway_tx, &tree, focused)?;
            }
            _ => {}
        }
    }
    Ok(())
}


fn set_workspace_name(
    sway: &mut swayipc::Connection,
    tree: &swayipc::Node,
    win: &swayipc::Node,
) -> Result<(), Box<dyn std::error::Error>> {

    let ws = find_workspace(tree, &win);
    let parent = find_parent(tree, &win);
    let siblings = parent.nodes.len();

    let win_name = win.app_id.clone()
        .or_else(|| win.window_properties.clone().and_then(|p| p.class.or(p.instance).clone()))
        .unwrap_or_default();

    let mut layout_icons = String::new();

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
                ""
            }
        }
    };

    let layout_icon_style = "font_size='16pt' color='lightgreen'";
    let layout_icon = format!("<span {layout_icon_style}>{layout_icon}</span>");

    let ws_old_name = ws.name.as_ref().expect("Unnamed workspace");
    let ws_num = ws.num.expect("Unnumbered workspace");

    let ws_icon_style = "baseline_shift='superscript' font_size='12pt' color='lightgreen'";
    let ws_icon = assign_icon(&win_name);

    let ws_name_style = "color='orange' baseline_shift='2pt'";
    let ws_name = if ws.id != win.id {
        format!(" {layout_icon} <span {ws_name_style}> {win_name} </span>")
    } else {
        String::new()
    };

    let ws_new_name = format!("{ws_num}<span {ws_icon_style}>{ws_icon}</span>{ws_name}");

    sway.run_command(format!("rename workspace {ws_old_name} to {ws_new_name}"))?;

    Ok(())
}


fn find_focused(tree: &swayipc::Node) -> &swayipc::Node {
    let mut stack = Vec::with_capacity(tree.nodes.len() + tree.floating_nodes.len());
    stack.push(tree);

    while let Some(n) = stack.pop() {
        if n.focused {
            return n
        }
        stack.extend(n.nodes.as_slice());
        stack.extend(n.floating_nodes.as_slice());
    }

    unreachable!("cannot find focused window in")
}

fn find_parent<'a>(tree: &'a swayipc::Node, win: &'a swayipc::Node) -> &'a swayipc::Node {
    let mut stack = Vec::with_capacity(tree.nodes.len() + tree.floating_nodes.len());
    stack.push(tree);

    while let Some(n) = stack.pop() {
        if n.nodes.iter().any(|n| n.id == win.id) {
            return n
        }
        if n.floating_nodes.iter().any(|n| n.id == win.id) {
            return n
        }
        stack.extend(n.nodes.as_slice());
        stack.extend(n.floating_nodes.as_slice());
    }

    unreachable!("cannot find parent for {}", win.id)
}

fn find_workspace<'a>(outputs: &'a swayipc::Node, win: &'a swayipc::Node) -> &'a swayipc::Node {
    let workspaces: Vec<_> = outputs.nodes.iter()
        .flat_map(|outputs| outputs.nodes.as_slice())
        .filter(|workspace| workspace.name.as_deref() != Some("__i3_scratch"))
        .collect();

    for ws in &workspaces {
        if ws.id == win.id {
            return ws
        }

        let mut stack = Vec::with_capacity(ws.nodes.len() + ws.floating_nodes.len());
        stack.extend(ws.nodes.as_slice());
        stack.extend(ws.floating_nodes.as_slice());

        while let Some(ws_win) = stack.pop() {
            if ws_win.id == win.id {
                return ws
            }
            stack.extend(ws_win.nodes.as_slice());
            stack.extend(ws_win.floating_nodes.as_slice());
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

