use swayipc::{Event, EventType, WindowEvent, NodeType, NodeLayout, WindowChange, Node, Connection};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let icons = get_icons();

    let sway_rx = Connection::new()?;

    let events = sway_rx
        .subscribe([EventType::Window, EventType::Binding])?;

    for e in events {
        match e {

            Ok(Event::Window(e)) if matches!(*e, WindowEvent {
                change: WindowChange::Focus | WindowChange::Move | WindowChange::Floating,
                ..
            }) => {
                let mut sway = Connection::new()?;
                let workspaces = get_workspaces(&mut sway)?;
                set_workspace_name(&mut sway, &workspaces, &e.container, &icons)?;
            },

            Ok(Event::Window(e)) if matches!(*e, WindowEvent {
                change: WindowChange::Close,
                ..
            }) => {
                let mut sway = Connection::new()?;
                let tree = get_workspaces(&mut sway)?;
                let focused = find_focused(&tree);
                set_workspace_name(&mut sway, &tree, focused, &icons)?;
            },

            Ok(Event::Binding(_)) => {
                let mut sway = Connection::new()?;
                let tree = get_workspaces(&mut sway)?;
                let focused = find_focused(&tree);
                set_workspace_name(&mut sway, &tree, focused, &icons)?;
            }

            _ => {}
        }
    }

    Ok(())
}

fn get_icons() -> Option<toml::value::Table> {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(|xdg_config_home| {
            std::path::PathBuf::from(xdg_config_home)
                .join("namedworkspaces")
        });

    let config_dir = config_dir.or_else(|_| std::env::var("HOME")
        .map(|home| {
            std::path::PathBuf::from(home)
                .join(".config/namedworkspaces")
        }));

    let config_dir = config_dir
        .map(|d| d.join("config.toml"));

    let Ok(config) = config_dir else {
        return None
    };

    let content = std::fs::read_to_string(config)
        .expect("cannot read config file");
    let value: toml::Value = toml::from_str(&content)
        .expect("invalid config format");

    value.get("applications")
        .and_then(|val| val.as_table())
        .cloned()
}


fn set_workspace_name(
    sway: &mut Connection,
    workspaces: &Vec<Node>,
    win: &Node,
    icons: &Option<toml::value::Table>
) -> Result<(), Box<dyn std::error::Error>> {

    let ws = find_workspace(workspaces, &win);
    let parent = find_parent(workspaces, &win);
    let siblings = parent.nodes.len();

    let win_properties = win.window_properties.as_ref();
    let win_name = win.app_id.as_deref()
        .or_else(|| win_properties.and_then(|p| p.class.as_deref().or(p.instance.as_deref())))
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
                        layout_icons += if parent.nodes[x].id == win.id { "▮" } else { "▯" };
                    }
                    &layout_icons
                }
            } else if parent.layout == NodeLayout::SplitV {
                if siblings == 2 {
                    if parent.nodes[0].id == win.id { "⬒" } else { "⬓" }
                } else {
                    "<span font_size='24pt' baseline_shift='-3pt'></span>"
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
    let ws_icon = assign_icon(&win_name, icons);

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

fn assign_icon<'a>(win_name: &str, icons: &'a Option<toml::value::Table>) -> &'a str {
    if win_name == "" {
        return "＋"
    }
    let Some(icons) = icons else {
        return ""
    };
    icons[win_name].as_str().unwrap_or("?")
}


fn get_workspaces(sway: &mut Connection) -> Result<Vec<Node>, Box<dyn std::error::Error>> {
    let tree = sway.get_tree()?;
    let workspaces = tree.nodes.into_iter().chain(tree.floating_nodes)
        .flat_map(|outputs| outputs.nodes.into_iter().chain(outputs.floating_nodes))
        .filter(|workspace| workspace.name.as_deref() != Some("__i3_scratch"))
        .collect();
    Ok(workspaces)
}


fn find_focused(workspaces: &Vec<Node>) -> &Node {
    let mut stack = Vec::with_capacity(workspaces.len());
    stack.extend(workspaces);

    while let Some(n) = stack.pop() {
        if n.focused {
            return n
        }
        stack.extend(n.nodes.as_slice());
        stack.extend(n.floating_nodes.as_slice());
    }

    unreachable!("cannot find focused window in")
}

fn find_parent<'a>(workspaces: &'a Vec<Node>, win: &'a Node) -> &'a Node {
    let mut stack = Vec::with_capacity(workspaces.len());
    stack.extend(workspaces);

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

fn find_workspace<'a>(workspaces: &'a Vec<Node>, win: &'a Node) -> &'a Node {
    for ws in workspaces {
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

