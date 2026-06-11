use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_query_map;

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

#[component]
pub fn SceneCreationPage() -> impl IntoView {
    let scene = query_value("scene").unwrap_or_else(|| "presentation".to_string());
    let selected_goal = RwSignal::new("改成口播".to_string());
    let content = RwSignal::new(String::new());
    let validation_message = RwSignal::new(String::new());

    let scene_for_next = scene.clone();
    let next_href = move || {
        format!(
            "/voice-style?scene={}&goal={}&text={}",
            encode_query_component(&scene_for_next),
            encode_query_component(&selected_goal.get()),
            encode_query_component(content.get().trim())
        )
    };

    view! {
        <div class="min-h-screen bg-gray-50/40 pb-16">
            <div class="container mx-auto max-w-[1180px] px-4 py-8 md:py-12">
                <StepHeader active=2 title="场景创作" subtitle="先把要表达的内容整理成适合生成声音的创作任务" />

                <div class="mt-8 grid gap-6 lg:grid-cols-[1fr_340px]">
                    <section class="rounded-[2rem] border border-amber-100 bg-white p-6 shadow-sm md:p-8">
                        <div class="flex flex-wrap items-center justify-between gap-4">
                            <div>
                                <p class="text-sm font-semibold text-amber-600">"Step 02 · "{scene_label(&scene)}</p>
                                <h1 class="mt-2 text-2xl font-bold text-gray-900 md:text-3xl">"写下这次声音要完成的事"</h1>
                                <p class="mt-3 max-w-2xl text-sm leading-7 text-gray-500">
                                    "不用一开始就写完整文稿，可以先选一个目标，再补充要点。下一步会继续选择声音滤镜。"
                                </p>
                            </div>
                            <A href="/home" attr:class="rounded-full border border-gray-200 bg-white px-4 py-2 text-sm font-semibold text-gray-500 hover:border-amber-200 hover:text-amber-600">
                                <i class="fa-solid fa-arrow-left mr-2"></i>"重选场景"
                            </A>
                        </div>

                        <div class="mt-8 grid gap-4 md:grid-cols-3">
                            <CreateGoalCard icon="fa-list-check" title="整理提纲" text="把几条想法变成清晰结构" color="emerald" selected_goal=selected_goal />
                            <CreateGoalCard icon="fa-comment-dots" title="改成口播" text="让文字更像自然说话" color="blue" selected_goal=selected_goal />
                            <CreateGoalCard icon="fa-clock" title="控制节奏" text="按时长和听众调整密度" color="amber" selected_goal=selected_goal />
                        </div>

                        <div class="mt-8 rounded-3xl border border-gray-100 bg-gray-50/80 p-5">
                            <label class="text-sm font-bold text-gray-800">"创作内容"</label>
                            <textarea
                                class="mt-3 min-h-[170px] w-full resize-none rounded-2xl border border-gray-200 bg-white p-4 text-sm leading-7 text-gray-700 outline-none transition focus:border-amber-300 focus:ring-4 focus:ring-amber-100"
                                placeholder="例如：我想做一段课程汇报开场，内容包括研究背景、核心发现和下一步计划……"
                                prop:value=move || content.get()
                                on:input=move |ev| {
                                    content.set(event_target_value(&ev));
                                    validation_message.set(String::new());
                                }
                            ></textarea>
                            <div class="mt-4 flex flex-wrap gap-3 text-xs text-gray-500">
                                <span class="rounded-full bg-white px-3 py-1.5 border border-gray-100">"可先输入关键词"</span>
                                <span class="rounded-full bg-white px-3 py-1.5 border border-gray-100">"也可以粘贴完整文稿"</span>
                                <span class="rounded-full bg-white px-3 py-1.5 border border-gray-100">"生成前还可以继续修改"</span>
                            </div>
                        </div>

                        <div class="mt-8 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                            <p class="text-sm text-gray-500">
                                {move || if validation_message.get().is_empty() { "确认内容方向后，继续为这段内容选择声音滤镜。".to_string() } else { validation_message.get() }}
                            </p>
                            <a href=next_href class="inline-flex items-center justify-center rounded-2xl bg-gradient-to-r from-amber-400 to-orange-400 px-6 py-3 font-bold text-white shadow-sm transition hover:-translate-y-0.5 hover:shadow-md"
                                on:click=move |ev| {
                                    if content.get().trim().is_empty() {
                                        ev.prevent_default();
                                        validation_message.set("请先写一点创作内容，这样后面的生成页才能直接使用。".to_string());
                                    }
                                }
                            >
                                "下一步：选择声音滤镜" <i class="fa-solid fa-arrow-right ml-2"></i>
                            </a>
                        </div>
                    </section>

                    <aside class="space-y-4">
                        <div class="rounded-[2rem] border border-white bg-white/90 p-6 shadow-sm">
                            <h2 class="font-bold text-gray-900">"推荐写法"</h2>
                            <div class="mt-4 space-y-3">
                                <TipItem title="说给谁听" text="同学、老师、观众或客户" />
                                <TipItem title="想达到什么效果" text="讲清楚、打动人、介绍产品或营造氛围" />
                                <TipItem title="希望多长" text="30 秒、1 分钟或 3 分钟都可以先写出来" />
                            </div>
                        </div>
                        <div class="rounded-[2rem] border border-emerald-100 bg-emerald-50/70 p-6">
                            <p class="text-sm font-semibold text-emerald-700">"普通用户"</p>
                            <p class="mt-2 text-sm leading-7 text-gray-600">"按当前页面一步步填，最后进入生成器即可。"</p>
                        </div>
                        <div class="rounded-[2rem] border border-amber-100 bg-amber-50/80 p-6">
                            <p class="text-sm font-semibold text-amber-700">"创作者 / 专业用户"</p>
                            <p class="mt-2 text-sm leading-7 text-gray-600">"可以在生成页继续调声线、语速、音量和更多参数。"</p>
                        </div>
                    </aside>
                </div>
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

    let scene_for_generate = scene.clone();
    let goal_for_generate = goal.clone();
    let text_for_generate = text.clone();
    let generate_href = move || {
        let style = selected_style.get();
        let (pitch, speed, volume) = match style.as_str() {
            "bright" => ("1.06", "1.12", "1.0"),
            "emotional" => ("1.08", "0.96", "1.08"),
            "steady" => ("0.94", "0.92", "1.0"),
            _ => ("1.0", "1.0", "1.0"),
        };
        let instruction = style_instruction(&style, &scene_for_generate, &goal_for_generate);

        format!(
            "/generate?scene={}&goal={}&style={}&text={}&instruction={}&pitch={}&speed={}&volume={}",
            encode_query_component(&scene_for_generate),
            encode_query_component(&goal_for_generate),
            encode_query_component(&style),
            encode_query_component(&text_for_generate),
            encode_query_component(&instruction),
            pitch,
            speed,
            volume
        )
    };

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
                        {move || if text.trim().is_empty() { "还没有带入文本，请返回补充。".to_string() } else { text.clone() }}
                    </div>

                    <div class="mt-8 grid gap-4 md:grid-cols-2 xl:grid-cols-4">
                        <VoiceStyleCard id="clear" icon="fa-sun" title="清晰讲解" text="适合汇报、课程、知识讲解" color="emerald" selected_style=selected_style />
                        <VoiceStyleCard id="bright" icon="fa-bolt" title="轻快口播" text="适合短视频、直播、信息流内容" color="blue" selected_style=selected_style />
                        <VoiceStyleCard id="emotional" icon="fa-masks-theater" title="情绪配音" text="适合剧情、角色、娱乐表达" color="purple" selected_style=selected_style />
                        <VoiceStyleCard id="steady" icon="fa-gem" title="专业沉稳" text="适合商务、学术、正式说明" color="amber" selected_style=selected_style />
                    </div>

                    <div class="mt-8 rounded-3xl border border-gray-100 bg-gray-50/80 p-5">
                        <h2 class="font-bold text-gray-900">"更多滤镜"</h2>
                        <p class="mt-2 text-sm leading-7 text-gray-500">
                            "如果想查看社区里已经发布的滤镜，可以先去声音滤镜库；如果已经确定方向，就直接进入生成器。"
                        </p>
                        <div class="mt-5 flex flex-col gap-3 sm:flex-row">
                            <A href="/filters" attr:class="inline-flex items-center justify-center rounded-2xl border border-gray-200 bg-white px-5 py-3 font-semibold text-gray-700 transition hover:border-amber-200 hover:text-amber-600">
                                <i class="fa-solid fa-layer-group mr-2"></i>"查看滤镜库"
                            </A>
                            <a href=generate_href class="inline-flex items-center justify-center rounded-2xl bg-gradient-to-r from-amber-400 to-orange-400 px-6 py-3 font-bold text-white shadow-sm transition hover:-translate-y-0.5 hover:shadow-md">
                                "进入声音生成" <i class="fa-solid fa-arrow-right ml-2"></i>
                            </a>
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
fn CreateGoalCard(
    icon: &'static str,
    title: &'static str,
    text: &'static str,
    color: &'static str,
    selected_goal: RwSignal<String>,
) -> impl IntoView {
    let classes = match color {
        "emerald" => (
            "border-emerald-100 bg-emerald-50/60 text-emerald-600",
            "bg-white",
        ),
        "blue" => ("border-blue-100 bg-blue-50/60 text-blue-600", "bg-white"),
        _ => ("border-amber-100 bg-amber-50/70 text-amber-600", "bg-white"),
    };

    view! {
        <button
            type="button"
            class=move || format!(
                "rounded-3xl border p-5 text-left transition hover:-translate-y-0.5 hover:shadow-md {} {}",
                classes.0,
                if selected_goal.get() == title { "ring-4 ring-amber-100 border-amber-300" } else { "" }
            )
            on:click=move |_| selected_goal.set(title.to_string())
        >
            <div class=format!("flex h-11 w-11 items-center justify-center rounded-2xl shadow-sm {}", classes.1)>
                <i class=format!("fa-solid {} text-xl", icon)></i>
            </div>
            <h3 class="mt-4 font-bold text-gray-900">{title}</h3>
            <p class="mt-2 text-sm text-gray-500">{text}</p>
        </button>
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
            on:click=move |_| selected_style.set(id.to_string())
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

#[component]
fn TipItem(title: &'static str, text: &'static str) -> impl IntoView {
    view! {
        <div class="rounded-2xl border border-gray-100 bg-gray-50/70 p-4">
            <p class="text-sm font-semibold text-gray-900">{title}</p>
            <p class="mt-1 text-xs text-gray-500">{text}</p>
        </div>
    }
}
