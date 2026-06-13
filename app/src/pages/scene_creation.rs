use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_query_map};

use crate::api;
use crate::pages::component::ai_rewrite_text;

fn query_value(key: &'static str) -> Option<String> {
    let query = use_query_map();
    query.with_untracked(|q| q.get(key).map(|value| value.to_string()))
}

fn encode_query_component(value: &str) -> String {
    let mut encoded = String::new();

    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }

    encoded
}

fn scene_label(scene: &str) -> &'static str {
    match scene {
        "content" => "内容创作",
        "dubbing" => "娱乐 / 配音",
        "expression" => "表达提升",
        "professional" => "专业表达",
        "custom" => "自定义场景",
        _ => "汇报 / 演讲",
    }
}

fn style_instruction(style: &str, scene: &str, goal: &str) -> String {
    let style_text = match style {
        "bright" => "轻快自然，语速略快，适合短视频和内容口播",
        "emotional" => "情绪饱满，有角色感和画面感，适合配音表达",
        "steady" => "专业沉稳，吐字清楚，适合正式汇报和商务说明",
        _ => "清晰讲解，结构分明，节奏稳定，适合把信息讲清楚",
    };

    format!(
        "场景：{}；创作目标：{}；声音滤镜：{}。请用{}的方式朗读，保持自然、清楚、适合口播。",
        scene_label(scene),
        goal,
        style_text,
        style_text
    )
}

fn style_params(style: &str) -> api::voice::Parametic {
    match style {
        "bright" => api::voice::Parametic {
            pitch: 1.06,
            speed: 1.12,
            volume: 1.0,
        },
        "emotional" => api::voice::Parametic {
            pitch: 1.08,
            speed: 0.96,
            volume: 1.08,
        },
        "steady" => api::voice::Parametic {
            pitch: 0.94,
            speed: 0.92,
            volume: 1.0,
        },
        _ => api::voice::Parametic {
            pitch: 1.0,
            speed: 1.0,
            volume: 1.0,
        },
    }
}

fn style_title(style: &str) -> &'static str {
    match style {
        "bright" => "轻快口播",
        "emotional" => "情绪配音",
        "steady" => "专业沉稳",
        _ => "清晰讲解",
    }
}

const DEFAULT_SCENE_EXAMPLE: &str = "• 全球航运发展趋势\n• AI 在航运中的应用\n• 未来发展方向";
const DEFAULT_AUDIENCE: &str = "老师 / 评委";
const DEFAULT_DURATION: &str = "3分钟";
const DEFAULT_INPUT_MODE: &str = "PPT 要点输入";
const DEFAULT_GOAL: &str = "帮我写成能讲的内容";

fn outline_points(content: &str) -> Vec<String> {
    let points: Vec<String> = content
        .lines()
        .filter_map(|line| {
            let trimmed = line
                .trim()
                .trim_start_matches(|c| matches!(c, '•' | '-' | '*' | '·'))
                .trim();

            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .collect();

    if points.is_empty() {
        vec!["请先输入几个要点，方便我帮你整理成能讲的话。".to_string()]
    } else {
        points
    }
}

fn section_title(index: usize, point: &str) -> String {
    match index {
        0 => format!("第一部分 {}", point),
        1 => format!("第二部分 {}", point),
        2 => format!("第三部分 {}", point),
        3 => format!("第四部分 {}", point),
        _ => point.to_string(),
    }
}

fn build_scene_draft(
    content: &str,
    audience: &str,
    duration: &str,
    input_mode: &str,
    goal: &str,
) -> String {
    let points = outline_points(content);
    let goal_hint = match goal {
        "让它更像人说的话" => "我会把语气写得更自然一点，像平时直接讲出来的话。",
        "解释专业内容" => "我会先解释专业概念，再补上例子，方便听众快速理解。",
        _ => "我会把要点整理成适合直接口播的表达。",
    };
    let input_hint = match input_mode {
        "一句话想法" => "现在你给的是一句话想法，我会先扩成完整表达，再帮你补结构。",
        "粘贴原稿优化" => "我会先保留原意，再把原稿整理得更顺、更适合说出来。",
        _ => "现在这组内容适合用 PPT 要点输入的方式继续展开。",
    };

    let mut draft = String::new();
    draft.push_str("开头（引入）\n");
    draft.push_str(&format!(
        "大家好，今天我会面向{}，用大约{}把这部分内容讲清楚。{} {}\n",
        audience, duration, goal_hint, input_hint
    ));

    for (index, point) in points.iter().enumerate() {
        draft.push_str("\n");
        draft.push_str(&section_title(index, point));
        draft.push_str("\n");
        draft.push_str(&format!(
            "围绕“{}”，我会先说核心结论，再补一个具体说明，让内容更容易听懂。",
            point
        ));
    }

    draft.push_str("\n\n结尾（总结）\n");
    draft.push_str(&format!(
        "综上，这段内容适合用{}的方式讲给{}听。后面如果你要，我还可以继续把它改得更像真实口播。",
        duration, audience
    ));

    draft
}

fn refresh_scene_draft(
    content: RwSignal<String>,
    selected_audience: RwSignal<String>,
    selected_duration: RwSignal<String>,
    selected_input_mode: RwSignal<String>,
    selected_goal: RwSignal<String>,
    optimized_text: RwSignal<String>,
) {
    optimized_text.set(build_scene_draft(
        &content.get(),
        &selected_audience.get(),
        &selected_duration.get(),
        &selected_input_mode.get(),
        &selected_goal.get(),
    ));
}

fn build_ai_prompt(
    scene_label: &str,
    audience: &str,
    duration: &str,
    input_mode: &str,
    goal: &str,
    user_content: &str,
) -> String {
    format!(
        "场景：{}\n受众：{}\n时长：{}\n输入方式：{}\n任务目标：{}\n\n用户原始内容：\n{}\n\n请根据以上信息，只输出一段可直接朗读/编辑的表达稿，不要解释、不要问候、不要额外说明。",
        scene_label, audience, duration, input_mode, goal, user_content
    )
}

#[component]
pub fn SceneCreationPage() -> impl IntoView {
    let scene = query_value("scene").unwrap_or_else(|| "presentation".to_string());
    let selected_goal = RwSignal::new(DEFAULT_GOAL.to_string());
    let selected_audience = RwSignal::new(DEFAULT_AUDIENCE.to_string());
    let selected_duration = RwSignal::new(DEFAULT_DURATION.to_string());
    let selected_input_mode = RwSignal::new(DEFAULT_INPUT_MODE.to_string());
    let content = RwSignal::new(DEFAULT_SCENE_EXAMPLE.to_string());
    let optimized_text = RwSignal::new(build_scene_draft(
        DEFAULT_SCENE_EXAMPLE,
        DEFAULT_AUDIENCE,
        DEFAULT_DURATION,
        DEFAULT_INPUT_MODE,
        DEFAULT_GOAL,
    ));
    let validation_message = RwSignal::new(String::new());
    let ai_pending = RwSignal::new(false);

    let ai_action = Action::new(move |input: &String| {
        let input = input.clone();
        async move { ai_rewrite_text(input).await }
    });

    Effect::new(move |_| {
        if let Some(Ok(result)) = ai_action.value().get() {
            optimized_text.set(result);
            ai_pending.set(false);
        } else if let Some(Err(err)) = ai_action.value().get() {
            validation_message.set(format!("AI 生成失败：{}", err));
            ai_pending.set(false);
        }
    });

    let scene_for_next = scene.clone();
    let scene_for_ai = scene.clone();
    let next_href = move || {
        let final_goal = format!(
            "{}｜受众：{}｜时长：{}｜输入方式：{}",
            selected_goal.get(),
            selected_audience.get(),
            selected_duration.get(),
            selected_input_mode.get()
        );

        format!(
            "/voice-style?scene={}&goal={}&text={}",
            encode_query_component(&scene_for_next),
            encode_query_component(&final_goal),
            encode_query_component(optimized_text.get().trim())
        )
    };

    let dispatch_ai_for_goal = {
        let dispatch = move |goal: &str| {
            if content.get().trim().is_empty() {
                validation_message.set("请先输入内容，再点击生成。".to_string());
                return;
            }
            ai_pending.set(true);
            validation_message.set(String::new());
            let prompt = build_ai_prompt(
                scene_label(&scene_for_ai),
                &selected_audience.get(),
                &selected_duration.get(),
                &selected_input_mode.get(),
                goal,
                &content.get(),
            );
            ai_action.dispatch(prompt);
        };
        std::rc::Rc::new(dispatch)
    };

    view! {
        <div class="min-h-screen bg-gray-50/40 pb-16">
            <div class="container mx-auto max-w-[1180px] px-4 py-8 md:py-12">
                <StepHeader active=2 title="场景创作" subtitle="先把要表达的内容整理成适合生成声音的创作任务" />

                <section class="mt-8 overflow-hidden rounded-[1.75rem] border border-gray-200 bg-white shadow-sm">
                    <div class="flex flex-wrap items-center justify-between gap-3 border-b border-gray-100 bg-white px-5 py-3">
                        <div class="inline-flex items-center gap-2 rounded-xl border border-amber-200 bg-amber-50 px-3 py-1.5 text-sm font-bold text-gray-800">
                            <span class="flex h-6 w-6 items-center justify-center rounded-full bg-amber-400 text-xs text-white">"2"</span>
                            "进入场景创作页（以“汇报 / 演讲”为例）"
                        </div>
                        <A href="/home" attr:class="rounded-full border border-gray-200 bg-white px-4 py-2 text-sm font-semibold text-gray-500 hover:border-emerald-200 hover:text-emerald-600">
                            <i class="fa-solid fa-arrow-left mr-2"></i>"重选场景"
                        </A>
                    </div>

                    <div class="grid lg:grid-cols-[220px_1fr]">
                        <aside class="border-b border-gray-100 bg-white p-5 lg:border-b-0 lg:border-r">
                            <div class="flex items-center gap-2">
                                <span class="flex h-7 w-7 items-center justify-center rounded-full bg-blue-100 text-xs text-blue-600">
                                    <i class="fa-solid fa-headset"></i>
                                </span>
                                <h2 class="font-bold text-gray-900">"表达助手"</h2>
                                <span class="text-amber-500">"✨"</span>
                            </div>

                            <div class="mt-6 space-y-6 text-sm">
                                <div>
                                    <p class="font-semibold text-gray-500">"场景：" <span class="font-bold text-gray-800">{scene_label(&scene)}</span></p>
                                </div>

                                <div>
                                    <label class="mb-2 block font-semibold text-gray-700">"受众"</label>
                                    <select
                                        class="w-full rounded-lg border border-gray-200 bg-white px-3 py-2 text-left text-gray-700 shadow-sm outline-none transition focus:border-emerald-300 focus:ring-4 focus:ring-emerald-100"
                                        prop:value=move || selected_audience.get()
                                        on:change=move |ev| {
                                            selected_audience.set(event_target_value(&ev));
                                            refresh_scene_draft(content, selected_audience, selected_duration, selected_input_mode, selected_goal, optimized_text);
                                        }
                                    >
                                        <option value="老师 / 评委">"老师 / 评委"</option>
                                        <option value="同学 / 观众">"同学 / 观众"</option>
                                        <option value="客户 / 用户">"客户 / 用户"</option>
                                    </select>
                                </div>

                                <div>
                                    <label class="mb-2 block font-semibold text-gray-700">"时长"</label>
                                    <div class="grid grid-cols-3 overflow-hidden rounded-lg border border-gray-200 bg-white text-xs font-semibold">
                                        <button
                                            type="button"
                                            class=move || if selected_duration.get() == "1分钟" { "bg-emerald-500 px-2 py-2 text-white" } else { "px-2 py-2 text-gray-500 hover:bg-gray-50" }
                                            on:click=move |_| {
                                                selected_duration.set("1分钟".to_string());
                                                refresh_scene_draft(content, selected_audience, selected_duration, selected_input_mode, selected_goal, optimized_text);
                                            }
                                        >
                                            "1分钟"
                                        </button>
                                        <button
                                            type="button"
                                            class=move || if selected_duration.get() == "3分钟" { "bg-emerald-500 px-2 py-2 text-white" } else { "px-2 py-2 text-gray-500 hover:bg-gray-50" }
                                            on:click=move |_| {
                                                selected_duration.set("3分钟".to_string());
                                                refresh_scene_draft(content, selected_audience, selected_duration, selected_input_mode, selected_goal, optimized_text);
                                            }
                                        >
                                            "3分钟"
                                        </button>
                                        <button
                                            type="button"
                                            class=move || if selected_duration.get() == "5分钟" { "bg-emerald-500 px-2 py-2 text-white" } else { "px-2 py-2 text-gray-500 hover:bg-gray-50" }
                                            on:click=move |_| {
                                                selected_duration.set("5分钟".to_string());
                                                refresh_scene_draft(content, selected_audience, selected_duration, selected_input_mode, selected_goal, optimized_text);
                                            }
                                        >
                                            "5分钟"
                                        </button>
                                    </div>
                                </div>

                                <div>
                                    <label class="mb-2 block font-semibold text-gray-700">"输入方式"</label>
                                    <div class="space-y-2">
                                        <button type="button" class=move || if selected_input_mode.get() == "PPT 要点输入" { "flex w-full items-center gap-2 rounded-lg border border-emerald-300 bg-emerald-50 px-3 py-2 text-left font-semibold text-emerald-700" } else { "flex w-full items-center gap-2 rounded-lg border border-gray-200 bg-white px-3 py-2 text-left text-gray-500 hover:bg-gray-50" } on:click=move |_| { selected_input_mode.set("PPT 要点输入".to_string()); if content.get().trim().is_empty() { content.set(DEFAULT_SCENE_EXAMPLE.to_string()); } refresh_scene_draft(content, selected_audience, selected_duration, selected_input_mode, selected_goal, optimized_text); }>
                                            <i class="fa-solid fa-pen text-xs"></i>"PPT 要点输入"
                                        </button>
                                        <button type="button" class=move || if selected_input_mode.get() == "一句话想法" { "flex w-full items-center gap-2 rounded-lg border border-emerald-300 bg-emerald-50 px-3 py-2 text-left font-semibold text-emerald-700" } else { "flex w-full items-center gap-2 rounded-lg border border-gray-200 bg-white px-3 py-2 text-left text-gray-500 hover:bg-gray-50" } on:click=move |_| { selected_input_mode.set("一句话想法".to_string()); if content.get().trim().is_empty() || content.get() == DEFAULT_SCENE_EXAMPLE { content.set("我想做一段三分钟的汇报，介绍全球航运发展趋势和 AI 在航运中的应用。".to_string()); } selected_goal.set("让它更像人说的话".to_string()); refresh_scene_draft(content, selected_audience, selected_duration, selected_input_mode, selected_goal, optimized_text); }>
                                            <i class="fa-regular fa-comment-dots text-xs"></i>"一句话想法"
                                        </button>
                                        <button type="button" class=move || if selected_input_mode.get() == "粘贴原稿优化" { "flex w-full items-center gap-2 rounded-lg border border-emerald-300 bg-emerald-50 px-3 py-2 text-left font-semibold text-emerald-700" } else { "flex w-full items-center gap-2 rounded-lg border border-gray-200 bg-white px-3 py-2 text-left text-gray-500 hover:bg-gray-50" } on:click=move |_| { selected_input_mode.set("粘贴原稿优化".to_string()); if content.get().trim().is_empty() || content.get() == DEFAULT_SCENE_EXAMPLE { content.set("近年来，全球航运市场保持稳步增长，AI 技术也开始在航线规划、船舶监测和港口运营里发挥作用。".to_string()); } selected_goal.set("解释专业内容".to_string()); refresh_scene_draft(content, selected_audience, selected_duration, selected_input_mode, selected_goal, optimized_text); }>
                                            <i class="fa-regular fa-clipboard text-xs"></i>"粘贴原稿优化"
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </aside>

                        <div class="bg-gray-50/40 p-5">
                            <div class="rounded-2xl border border-gray-200 bg-white p-5 shadow-sm">
                                <div class="mb-3 flex items-center justify-between">
                                    <label class="text-sm font-bold text-gray-800">"输入你的要点 / PPT 内容（示例）"</label>
                                    <button
                                        type="button"
                                        class="text-xs font-semibold text-gray-400 hover:text-gray-700"
                                        on:click=move |_| {
                                            content.set(String::new());
                                            optimized_text.set(String::new());
                                            validation_message.set(String::new());
                                        }
                                    >
                                        <i class="fa-solid fa-eraser mr-1"></i>"清空"
                                    </button>
                                </div>
                                <textarea
                                    class="min-h-[126px] w-full resize-none rounded-xl border border-gray-200 bg-white p-4 text-sm leading-7 text-gray-700 outline-none transition focus:border-emerald-300 focus:ring-4 focus:ring-emerald-100"
                                    placeholder="例如：全球航运发展趋势、AI 在航运中的应用、未来发展方向……"
                                    prop:value=move || content.get()
                                    on:input=move |ev| {
                                        content.set(event_target_value(&ev));
                                        validation_message.set(String::new());
                                        refresh_scene_draft(content, selected_audience, selected_duration, selected_input_mode, selected_goal, optimized_text);
                                    }
                                ></textarea>
                                <div class="mt-4 flex flex-wrap gap-3">
                                    <button type="button" class=move || if selected_goal.get() == "帮我写成能讲的内容" { "rounded-lg border border-emerald-100 bg-emerald-500 px-4 py-2 text-sm font-bold text-white shadow-sm transition hover:bg-emerald-600 disabled:opacity-50 disabled:cursor-not-allowed" } else { "rounded-lg border border-emerald-100 bg-emerald-50 px-4 py-2 text-sm font-bold text-emerald-700 transition hover:bg-emerald-100 disabled:opacity-50 disabled:cursor-not-allowed" } disabled=move || ai_pending.get() on:click={ let d = dispatch_ai_for_goal.clone(); move |_| { selected_goal.set("帮我写成能讲的内容".to_string()); if content.get().trim().is_empty() || content.get() == DEFAULT_SCENE_EXAMPLE { content.set(DEFAULT_SCENE_EXAMPLE.to_string()); } d("帮我写成能讲的内容"); } }>
                                        <i class="fa-solid fa-wand-magic-sparkles mr-2"></i>{move || if ai_pending.get() && selected_goal.get() == "帮我写成能讲的内容" { "AI 生成中..." } else { "帮我写成能讲的内容" }}
                                    </button>
                                    <button type="button" class=move || if selected_goal.get() == "让它更像人说的话" { "rounded-lg border border-amber-100 bg-amber-500 px-4 py-2 text-sm font-bold text-white transition hover:bg-amber-600 disabled:opacity-50 disabled:cursor-not-allowed" } else { "rounded-lg border border-amber-100 bg-amber-50 px-4 py-2 text-sm font-bold text-amber-700 transition hover:bg-amber-100 disabled:opacity-50 disabled:cursor-not-allowed" } disabled=move || ai_pending.get() on:click={ let d = dispatch_ai_for_goal.clone(); move |_| { selected_goal.set("让它更像人说的话".to_string()); d("让它更像人说的话"); } }>
                                        <i class="fa-solid fa-microphone-lines mr-2"></i>{move || if ai_pending.get() && selected_goal.get() == "让它更像人说的话" { "AI 生成中..." } else { "让它更像人说的话" }}
                                    </button>
                                    <button type="button" class=move || if selected_goal.get() == "解释专业内容" { "rounded-lg border border-blue-100 bg-blue-500 px-4 py-2 text-sm font-bold text-white transition hover:bg-blue-600 disabled:opacity-50 disabled:cursor-not-allowed" } else { "rounded-lg border border-blue-100 bg-blue-50 px-4 py-2 text-sm font-bold text-blue-700 transition hover:bg-blue-100 disabled:opacity-50 disabled:cursor-not-allowed" } disabled=move || ai_pending.get() on:click={ let d = dispatch_ai_for_goal.clone(); move |_| { selected_goal.set("解释专业内容".to_string()); d("解释专业内容"); } }>
                                        <i class="fa-solid fa-book-open mr-2"></i>{move || if ai_pending.get() && selected_goal.get() == "解释专业内容" { "AI 生成中..." } else { "解释专业内容" }}
                                    </button>
                                </div>
                            </div>

                            <div class="mt-5 rounded-2xl border border-gray-200 bg-white p-5 shadow-sm">
                                <div class="mb-4 flex items-center justify-between border-b border-gray-100 pb-3">
                                    <label class="text-sm font-bold text-gray-800">"AI 优化后的表达（可编辑）"</label>
                                    <button type="button" class="text-xs font-bold text-emerald-600 hover:text-emerald-700 disabled:opacity-50 disabled:cursor-not-allowed" disabled=move || ai_pending.get() on:click={ let d = dispatch_ai_for_goal.clone(); move |_| { let g = selected_goal.get(); d(&g); } }>
                                        <i class="fa-solid fa-arrows-split-up-and-left mr-1"></i>{move || if ai_pending.get() { "AI 生成中..." } else { "AI 重新生成" }}
                                    </button>
                                </div>
                                <textarea
                                    class="min-h-[360px] w-full resize-none rounded-2xl border border-gray-200 bg-gray-50 p-4 text-sm leading-7 text-gray-700 outline-none transition focus:border-emerald-300 focus:ring-4 focus:ring-emerald-100"
                                    prop:value=move || optimized_text.get()
                                    on:input=move |ev| {
                                        optimized_text.set(event_target_value(&ev));
                                    }
                                ></textarea>
                            </div>

                            <div class="mt-5 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                                <p class="text-sm text-gray-500">
                                    {move || if validation_message.get().is_empty() { "确认这段表达后，继续选择声音滤镜。".to_string() } else { validation_message.get() }}
                                </p>
                                <a href=next_href class="inline-flex items-center justify-center rounded-xl bg-emerald-500 px-6 py-3 font-bold text-white shadow-sm transition hover:-translate-y-0.5 hover:bg-emerald-600 hover:shadow-md"
                                    on:click=move |ev| {
                                        if optimized_text.get().trim().is_empty() {
                                            ev.prevent_default();
                                            validation_message.set("请先输入或生成一段可用表达，这样后面的生成页才能直接使用。".to_string());
                                        }
                                    }
                                >
                                    <i class="fa-solid fa-circle-play mr-2"></i>"用这段生成声音" <i class="fa-solid fa-arrow-right ml-2"></i>
                                </a>
                            </div>
                        </div>
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
pub fn VoiceStylePage() -> impl IntoView {
    let scene = query_value("scene").unwrap_or_else(|| "presentation".to_string());
    let goal = query_value("goal").unwrap_or_else(|| "改成口播".to_string());
    let text = query_value("text").unwrap_or_default();
    let selected_style = RwSignal::new("clear".to_string());
    let selected_filter_meta_id = RwSignal::new(None::<String>);
    let selected_filter_title = RwSignal::new(String::new());
    let filter_message = RwSignal::new(String::new());
    let navigate = use_navigate();
    let navigate_after_create = navigate.clone();
    let navigate_existing_filter = navigate.clone();
    let scene_for_filter = scene.clone();
    let goal_for_filter = goal.clone();
    let text_for_filter = text.clone();

    let create_filter_action = Action::new(move |style: &String| {
        let style = style.clone();
        let scene = scene_for_filter.clone();
        let goal = goal_for_filter.clone();
        let text = text_for_filter.clone();
        async move {
            let voices = api::voice::list_voice_models().await?;
            let voice_model = voices
                .iter()
                .find(|voice| voice.ability.instruction_control)
                .or_else(|| voices.iter().find(|voice| voice.ability.parametric_control))
                .ok_or_else(|| {
                    ServerFnError::new("暂无支持指令或参数控制的声线，无法创建声音滤镜")
                })?;

            let fallback_parametric = style_params(&style);
            let fallback_instruction = style_instruction(&style, &scene, &goal);

            let parametric = voice_model
                .ability
                .parametric_control
                .then(|| fallback_parametric.clone());
            let instruction = voice_model
                .ability
                .instruction_control
                .then(|| fallback_instruction.clone());

            let voice_meta = api::voice::VoiceMeta {
                voice_model_id: voice_model.id,
                parametric,
                instruction: instruction.clone(),
            };

            let meta_id = api::voice::generate_meta(voice_meta).await?;
            let instruction_query = instruction
                .as_deref()
                .map(|instruction| format!("&instruction={}", encode_query_component(instruction)))
                .unwrap_or_default();

            Ok::<String, ServerFnError>(format!(
                "/generate?meta_id={}&text={}&pitch={:.3}&speed={:.3}&volume={:.3}{}#voice-selector",
                meta_id,
                encode_query_component(&text),
                fallback_parametric.pitch,
                fallback_parametric.speed,
                fallback_parametric.volume,
                instruction_query
            ))
        }
    });

    let filters_resource = Resource::new(
        || (),
        |_| async move {
            let mut filters =
                api::post::list_voice_meta_post(Some(api::post::PostStatus::Recommended), 6)
                    .await?;
            let mut normal_filters =
                api::post::list_voice_meta_post(Some(api::post::PostStatus::Normal), 6).await?;
            filters.append(&mut normal_filters);
            Ok::<Vec<api::post::VoiceMetaPost>, ServerFnError>(filters)
        },
    );

    let text_for_display = text.clone();
    let text_for_disabled = text.clone();
    let text_for_click = text.clone();

    Effect::new(move |_| {
        if let Some(result) = create_filter_action.value().get() {
            match result {
                Ok(url) => {
                    filter_message.set("已创建真实声音滤镜，正在进入生成页...".to_string());
                    navigate_after_create(&url, Default::default());
                }
                Err(err) => {
                    filter_message.set(format!("创建声音滤镜失败：{}", err));
                }
            }
        }
    });

    let back_href = format!("/scene-create?scene={}", encode_query_component(&scene));

    view! {
        <div class="min-h-screen bg-gray-50/40 pb-16">
            <div class="container mx-auto max-w-[1180px] px-4 py-8 md:py-12">
                <StepHeader active=3 title="声音滤镜" subtitle="为当前场景选择一个声音方向，再进入生成器完成试听" />

                <section class="mt-8 rounded-[2rem] border border-amber-100 bg-white p-6 shadow-sm md:p-8">
                    <div class="flex flex-wrap items-end justify-between gap-4">
                        <div>
                            <p class="text-sm font-semibold text-amber-600">"Step 03"</p>
                            <h1 class="mt-2 text-2xl font-bold text-gray-900 md:text-3xl">"选择这次作品的声音气质"</h1>
                        </div>
                        <A href=back_href attr:class="rounded-full border border-gray-200 bg-white px-4 py-2 text-sm font-semibold text-gray-500 hover:border-amber-200 hover:text-amber-600">
                            <i class="fa-solid fa-arrow-left mr-2"></i>"返回创作内容"
                        </A>
                    </div>

                    <div class="mt-6 rounded-3xl border border-amber-100 bg-amber-50/70 p-5 text-sm leading-7 text-gray-600">
                        <span class="font-bold text-amber-700">"当前内容："</span>
                        {move || if text_for_display.trim().is_empty() { "还没有带入文本，请返回补充。".to_string() } else { text_for_display.clone() }}
                    </div>

                    <div class="mt-8 grid gap-4 md:grid-cols-2 xl:grid-cols-4">
                        <VoiceStyleCard id="clear" icon="fa-sun" title="清晰讲解" text="适合汇报、课程、知识讲解" color="emerald" selected_style=selected_style selected_filter_meta_id=selected_filter_meta_id selected_filter_title=selected_filter_title />
                        <VoiceStyleCard id="bright" icon="fa-bolt" title="轻快口播" text="适合短视频、直播、信息流内容" color="blue" selected_style=selected_style selected_filter_meta_id=selected_filter_meta_id selected_filter_title=selected_filter_title />
                        <VoiceStyleCard id="emotional" icon="fa-masks-theater" title="情绪配音" text="适合剧情、角色、娱乐表达" color="purple" selected_style=selected_style selected_filter_meta_id=selected_filter_meta_id selected_filter_title=selected_filter_title />
                        <VoiceStyleCard id="steady" icon="fa-gem" title="专业沉稳" text="适合商务、学术、正式说明" color="amber" selected_style=selected_style selected_filter_meta_id=selected_filter_meta_id selected_filter_title=selected_filter_title />
                    </div>

                    <div class="mt-8 rounded-3xl border border-emerald-100 bg-emerald-50/50 p-5">
                        <div class="flex flex-wrap items-center justify-between gap-3">
                            <div>
                                <h2 class="font-bold text-gray-900">"已有真实声音滤镜"</h2>
                                <p class="mt-1 text-sm text-gray-500">"优先选择滤镜库里已经保存的真实 meta_id；没有合适的，再按上面的风格即时创建一个。"</p>
                            </div>
                            <A href="/filters" attr:class="rounded-full border border-emerald-200 bg-white px-4 py-2 text-sm font-semibold text-emerald-700 hover:bg-emerald-50">
                                <i class="fa-solid fa-layer-group mr-2"></i>"打开滤镜库"
                            </A>
                        </div>
                        <Suspense fallback=move || view! { <p class="mt-4 text-sm text-gray-500">"正在加载已有滤镜..."</p> }>
                            {move || match filters_resource.get() {
                                Some(Ok(filters)) if !filters.is_empty() => {
                                    let visible_filters: Vec<_> = filters.into_iter().take(4).collect();
                                    view! {
                                        <div class="mt-4 grid gap-3 md:grid-cols-2">
                                            <For
                                                each=move || visible_filters.clone()
                                                key=|filter| filter.id
                                                children=move |filter| {
                                                    let meta_id = filter.meta_id.to_string();
                                                    let title = filter.title.clone();
                                                    let content = filter.content.clone();
                                                    let is_recommended = matches!(filter.status.clone(), api::post::PostStatus::Recommended);
                                                    let meta_id_for_class = meta_id.clone();
                                                    let meta_id_for_click = meta_id.clone();
                                                    let title_for_click = title.clone();
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class=move || format!(
                                                                "rounded-2xl border bg-white p-4 text-left transition hover:-translate-y-0.5 hover:shadow-sm {}",
                                                                if selected_filter_meta_id.get().as_deref() == Some(meta_id_for_class.as_str()) { "border-emerald-400 ring-4 ring-emerald-100" } else { "border-emerald-100" }
                                                            )
                                                            on:click=move |_| {
                                                                selected_filter_meta_id.set(Some(meta_id_for_click.clone()));
                                                                selected_filter_title.set(title_for_click.clone());
                                                                filter_message.set(format!("已选择已有真实滤镜「{}」。", title_for_click));
                                                            }
                                                        >
                                                            <div class="flex items-center justify-between gap-3">
                                                                <h3 class="font-bold text-gray-900">{title}</h3>
                                                                <span class=if is_recommended { "rounded-full bg-amber-100 px-2 py-1 text-xs font-bold text-amber-700" } else { "rounded-full bg-gray-100 px-2 py-1 text-xs font-bold text-gray-500" }>
                                                                    {if is_recommended { "官方推荐" } else { "社区滤镜" }}
                                                                </span>
                                                            </div>
                                                            <p class="mt-2 line-clamp-2 text-sm leading-6 text-gray-500">{content}</p>
                                                            <p class="mt-3 text-xs font-semibold text-emerald-700">"meta_id: " {meta_id}</p>
                                                        </button>
                                                    }
                                                }
                                            />
                                        </div>
                                    }.into_any()
                                }
                                Some(Ok(_)) => view! { <p class="mt-4 rounded-2xl bg-white px-4 py-3 text-sm text-gray-500">"当前没有可直接使用的已发布滤镜。点击下方按钮会用你选择的风格创建新的真实 voice_meta。"</p> }.into_any(),
                                Some(Err(err)) => view! { <p class="mt-4 rounded-2xl bg-white px-4 py-3 text-sm text-red-500">"加载已有滤镜失败：" {err.to_string()}</p> }.into_any(),
                                None => view! { <p class="mt-4 text-sm text-gray-500">"正在加载已有滤镜..."</p> }.into_any(),
                            }}
                        </Suspense>
                    </div>

                    <div class="mt-8 rounded-3xl border border-gray-100 bg-gray-50/80 p-5">
                        <h2 class="font-bold text-gray-900">"更多滤镜"</h2>
                        <p class="mt-2 text-sm leading-7 text-gray-500">
                            "这里会优先使用你选择的声音方向创建真实 voice_meta 滤镜，再进入生成器；如果想挑选社区已发布的滤镜，也可以去滤镜库。"
                        </p>
                        <Show when=move || !filter_message.get().is_empty()>
                            <p class="mt-3 rounded-2xl bg-white px-4 py-3 text-sm font-semibold text-amber-700">
                                {move || filter_message.get()}
                            </p>
                        </Show>
                        <div class="mt-5 flex flex-col gap-3 sm:flex-row">
                            <A href="/filters" attr:class="inline-flex items-center justify-center rounded-2xl border border-gray-200 bg-white px-5 py-3 font-semibold text-gray-700 transition hover:border-amber-200 hover:text-amber-600">
                                <i class="fa-solid fa-layer-group mr-2"></i>"查看滤镜库"
                            </A>
                            <button
                                type="button"
                                class="inline-flex items-center justify-center rounded-2xl bg-gradient-to-r from-amber-400 to-orange-400 px-6 py-3 font-bold text-white shadow-sm transition hover:-translate-y-0.5 hover:shadow-md disabled:cursor-not-allowed disabled:opacity-60"
                                disabled=move || create_filter_action.pending().get() || text_for_disabled.trim().is_empty()
                                on:click=move |_| {
                                    if text_for_click.trim().is_empty() {
                                        filter_message.set("请先返回上一步生成或填写表达稿。".to_string());
                                        return;
                                    }

                                    if let Some(meta_id) = selected_filter_meta_id.get() {
                                        filter_message.set(format!("正在应用已有真实滤镜「{}」...", selected_filter_title.get()));
                                        navigate_existing_filter(
                                            &format!(
                                                "/generate?meta_id={}&text={}#voice-selector",
                                                meta_id,
                                                encode_query_component(&text_for_click)
                                            ),
                                            Default::default(),
                                        );
                                        return;
                                    }

                                    filter_message.set(format!("正在创建「{}」真实声音滤镜...", style_title(&selected_style.get())));
                                    create_filter_action.dispatch(selected_style.get());
                                }
                            >
                                {move || if create_filter_action.pending().get() { "正在创建滤镜...".to_string() } else if !selected_filter_title.get().is_empty() { format!("使用已有滤镜「{}」生成声音", selected_filter_title.get()) } else { format!("创建「{}」滤镜并生成声音", style_title(&selected_style.get())) }}
                                <i class="fa-solid fa-arrow-right ml-2"></i>
                            </button>
                        </div>
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
fn StepHeader(active: u8, title: &'static str, subtitle: &'static str) -> impl IntoView {
    let steps = [
        (1, "选择场景"),
        (2, "场景创作"),
        (3, "声音滤镜"),
        (4, "生成声音"),
    ];

    view! {
        <div>
            <p class="text-sm font-semibold text-amber-600">"EarDo 创作流程"</p>
            <h1 class="mt-2 text-3xl font-bold text-gray-900 md:text-4xl">{title}</h1>
            <p class="mt-3 text-gray-500">{subtitle}</p>
            <div class="mt-6 flex flex-wrap items-center gap-2">
                <For
                    each=move || steps
                    key=|step| step.0
                    children=move |(index, label)| {
                        let is_active = index == active;
                        view! {
                            <span class=if is_active { "inline-flex items-center gap-1.5 rounded-full border border-amber-200 bg-white px-3 py-1.5 text-sm font-semibold text-amber-700 shadow-sm" } else { "inline-flex items-center gap-1.5 rounded-full border border-gray-200 bg-white px-3 py-1.5 text-sm font-semibold text-gray-500 shadow-sm" }>
                                <span class=if is_active { "flex h-5 w-5 items-center justify-center rounded-full bg-amber-400 text-xs text-white" } else { "flex h-5 w-5 items-center justify-center rounded-full bg-gray-100 text-xs text-gray-400" }>{index}</span>
                                {label}
                            </span>
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
fn VoiceStyleCard(
    id: &'static str,
    icon: &'static str,
    title: &'static str,
    text: &'static str,
    color: &'static str,
    selected_style: RwSignal<String>,
    selected_filter_meta_id: RwSignal<Option<String>>,
    selected_filter_title: RwSignal<String>,
) -> impl IntoView {
    let classes = match color {
        "emerald" => {
            "border-emerald-100 bg-emerald-50/50 text-emerald-600 hover:border-emerald-300"
        }
        "blue" => "border-blue-100 bg-blue-50/50 text-blue-600 hover:border-blue-300",
        "purple" => "border-purple-100 bg-purple-50/50 text-purple-600 hover:border-purple-300",
        _ => "border-amber-100 bg-amber-50/60 text-amber-600 hover:border-amber-300",
    };

    view! {
        <button
            type="button"
            class=move || format!(
                "group rounded-3xl border p-5 text-left transition hover:-translate-y-0.5 hover:shadow-md {} {}",
                classes,
                if selected_style.get() == id { "ring-4 ring-amber-100 border-amber-300" } else { "" }
            )
            on:click=move |_| {
                selected_style.set(id.to_string());
                selected_filter_meta_id.set(None);
                selected_filter_title.set(String::new());
            }
        >
            <div class="flex h-12 w-12 items-center justify-center rounded-2xl bg-white shadow-sm transition group-hover:scale-105">
                <i class=format!("fa-solid {} text-xl", icon)></i>
            </div>
            <h3 class="mt-5 font-bold text-gray-900">{title}</h3>
            <p class="mt-2 text-sm leading-6 text-gray-500">{text}</p>
            <div class="mt-5 inline-flex items-center text-sm font-semibold">
                "选择此风格" <i class="fa-solid fa-check ml-2"></i>
            </div>
        </button>
    }
}
