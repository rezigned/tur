use serde_json::json;
use tur::{Program, Transition};
use yew::prelude::*;

#[derive(Properties, Clone)]
pub struct GraphViewProps {
    pub program: Program,
    pub current_state: String,
    pub previous_state: String,
    pub last_transition: Option<Transition>,
    pub step_count: usize,
}

impl PartialEq for GraphViewProps {
    fn eq(&self, other: &Self) -> bool {
        self.current_state == other.current_state
            && self.previous_state == other.previous_state
            && self.last_transition == other.last_transition
            && self.step_count == other.step_count
            && self.program == other.program
    }
}

pub struct GraphView {
    container_ref: NodeRef,
}

pub enum Msg {}

impl Component for GraphView {
    type Message = Msg;
    type Properties = GraphViewProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            container_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="graph-view">
                <div
                    ref={self.container_ref.clone()}
                    id="graph-container"
                    style="width: 100%; height: 400px; border: 1px solid #ccc; background: #f9f9f9; position: relative;"
                >
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            self.init_graph(ctx);
        }
        // Always update node styles on render to reflect current state
        self.update_node_styles(ctx);
    }

    fn changed(&mut self, ctx: &Context<Self>, old_props: &Self::Properties) -> bool {
        if ctx.props().program != old_props.program {
            self.init_graph(ctx); // Re-initialize if the program changes
        } else {
            self.update_node_styles(ctx);
        }

        if ctx.props().last_transition.is_some() && ctx.props().step_count != old_props.step_count {
            // Find the specific edge_id that corresponds to the last transition
            if let Some(last_trans) = &ctx.props().last_transition {
                let from_state = &ctx.props().previous_state;
                let to_state = &last_trans.next_state;

                // Iterate through the program rules to find the matching edge_id
                let mut found_edge_id: Option<String> = None;
                if let Some(transitions_for_state) = ctx.props().program.rules.get(from_state) {
                    for (i, transition) in transitions_for_state.iter().enumerate() {
                        // Compare the full transition struct to ensure exact match
                        if transition == last_trans {
                            found_edge_id = Some(format!("{}-{}-{}", from_state, to_state, i));
                            break;
                        }
                    }
                }

                if let Some(edge_id) = found_edge_id {
                    self.animate_state_transition(&edge_id);
                }
            }
        }

        true
    }
}

impl GraphView {
    fn get_graph_elements_json(&self, ctx: &Context<Self>) -> String {
        let props = ctx.props();
        let mut elements = Vec::new();

        // Add nodes
        let mut all_states = std::collections::HashSet::new();
        props.program.rules.keys().for_each(|s| {
            all_states.insert(s.clone());
        });
        props.program.rules.values().flatten().for_each(|t| {
            all_states.insert(t.next_state.clone());
        });
        if !props.current_state.is_empty() {
            all_states.insert(props.current_state.clone());
        }
        if all_states.is_empty() {
            all_states.insert("start".to_string());
        }

        for state in &all_states {
            let classes = String::new();
            // Initial classes, will be updated by update_node_styles
            elements.push(json!({
                "data": {
                    "id": state,
                },
                "classes": classes.trim()
            }));
        }

        // Add edges
        for (from_state, transitions) in &props.program.rules {
            for (i, transition) in transitions.iter().enumerate() {
                let to_state = &transition.next_state;
                let edge_id = format!("{}-{}-{}", from_state, to_state, i);
                let label = self.format_transition_label(transition);

                elements.push(json!({
                    "data": {
                        "id": edge_id,
                        "source": from_state,
                        "target": to_state,
                        "label": label
                    }
                }));
            }
        }

        serde_json::to_string(&elements).unwrap_or_default()
    }

    fn init_graph(&self, ctx: &Context<Self>) {
        let elements_json = self.get_graph_elements_json(ctx);
        let init_code = format!(
            r#"
            (function initGraphWithRetry() {{
                let retryCount = 0;
                const maxRetries = 10;

                function tryInit() {{
                    if (typeof cytoscape === 'undefined') {{
                        if (retryCount < maxRetries) {{
                            retryCount++;
                            setTimeout(tryInit, 200);
                        }}
                        return;
                    }}

                    if (typeof window.graphTheme === 'undefined') {{
                        if (retryCount < maxRetries) {{
                            retryCount++;
                            setTimeout(tryInit, 200);
                        }}
                        return;
                    }}

                    const container = document.getElementById('graph-container');
                    if (!container) {{
                        if (retryCount < maxRetries) {{
                            retryCount++;
                            setTimeout(tryInit, 200);
                        }}
                        return;
                    }}

                    try {{
                        const gt = window.graphTheme;

                        window.graphCy = cytoscape({{
                            container: container,
                            elements: JSON.parse(`{}`),
                            style: [
                                {{ selector: 'node', style: {{ 'background-color': gt.defaultNodeColor, 'label': 'data(id)', 'color': 'white', 'text-valign': 'center', 'text-halign': 'center', 'font-size': '14px', 'width': '50px', 'height': '50px', 'border-width': '0px' }} }},
                                {{ selector: '.current', style: {{ 'background-color': gt.activeNodeColor }} }},
                                {{ selector: '.previous', style: {{ 'background-color': gt.defaultNodeColor }} }},
                                {{ selector: '.halt', style: {{ 'background-color': gt.defaultNodeColor }} }},
                                {{ selector: 'edge', style: {{ 'width': 2, 'line-color': gt.edgeColor, 'target-arrow-color': gt.edgeColor, 'target-arrow-shape': 'triangle', 'curve-style': 'bezier', 'font-family': '"Fira Code", monospace', 'font-size': '12px', 'color': '#444', 'text-background-color': '#F5F7FA', 'text-background-opacity': 0.8 }} }},
                                {{ selector: 'edge[label]', style: {{ 'label': 'data(label)', 'text-wrap': 'wrap', 'text-max-width': '120px' }} }}
                            ],
                            layout: {{ name: 'circle', padding: 30 }},
                            ready: function() {{
                                this.fit(null, 20);
                                const startNode = this.getElementById('start');
                                if (startNode.length > 0) {{
                                    startNode.addClass('current');
                                }}
                            }}
                        }});
                    }} catch (error) {{
                        if (retryCount < maxRetries) {{
                            retryCount++;
                            setTimeout(tryInit, 500);
                        }}
                    }}
                }}

                tryInit();
            }})();
        "#,
            elements_json
        );
        let _ = js_sys::eval(&init_code);
    }

    fn update_node_styles(&self, ctx: &Context<Self>) {
        let props = ctx.props();
        let update_code = format!(
            r#"
            if (window.graphCy && window.graphTheme) {{
                const gt = window.graphTheme;
                const allNodes = window.graphCy.nodes();
                const previousNode = window.graphCy.getElementById('{}');
                const currentNode = window.graphCy.getElementById('{}');

                allNodes.stop(true, true).style({{
                    'background-color': gt.defaultNodeColor,
                    'border-width': '0px'
                }});
                allNodes.removeClass('current previous');

                if ('{}' !== '{}') {{
                    if (previousNode.length > 0) {{
                        previousNode.style({{
                            'background-color': gt.defaultNodeColor
                        }});
                        previousNode.addClass('previous');
                    }}
                    if (currentNode.length > 0) {{
                        currentNode.animate({{
                            style: {{ 'background-color': gt.activeNodeColor }},
                            duration: 400
                        }});
                        currentNode.addClass('current');
                    }}
                }} else {{
                    if (currentNode.length > 0) {{
                        currentNode.animate({{
                            style: {{ 'background-color': gt.activeNodeColor }},
                            duration: 400
                        }});
                        currentNode.addClass('current');
                    }}
                }}
            }}
        "#,
            props.previous_state, props.current_state, props.previous_state, props.current_state
        );
        let _ = js_sys::eval(&update_code);
    }

    fn animate_state_transition(&self, edge_id: &str) {
        let animate_code = format!(
            r#"
                if (window.graphCy && window.graphTheme) {{
                    const gt = window.graphTheme;
                    // Stop all ongoing animations on all edges and reset their styles
                    window.graphCy.edges().stop(true, true).style({{'line-color': gt.edgeColor, 'target-arrow-color': gt.edgeColor, 'width': 2}});

                    const edgeToAnimate = window.graphCy.getElementById('{}');

                    if (edgeToAnimate.length > 0) {{
                        edgeToAnimate.animate({{
                            style: {{'line-color': gt.edgeHighlightColor, 'target-arrow-color': gt.edgeHighlightColor, 'width': 4}},
                            duration: 1,
                            complete: function() {{
                                edgeToAnimate.animate({{
                                    style: {{'line-color': gt.edgeColor, 'target-arrow-color': gt.edgeColor, 'width': 2}},
                                    duration: 400
                                }});
                            }}
                        }});
                    }} else {{
                        console.warn(`Edge with ID '{}' not found for animation.`, edge_id);
                    }}
                }}
                "#,
            edge_id, edge_id
        );
        let _ = js_sys::eval(&animate_code);
    }

    fn format_transition_label(&self, transition: &Transition) -> String {
        let read_str: String = transition.read.iter().collect();
        let write_str: String = transition.write.iter().collect();
        let dir_str: String = transition
            .directions
            .iter()
            .map(|d| match d {
                tur::Direction::Left => "L",
                tur::Direction::Right => "R",
                tur::Direction::Stay => "S",
            })
            .collect::<Vec<&str>>()
            .join(",");

        format!("{} -> {}, {}", read_str, write_str, dir_str)
    }
}
