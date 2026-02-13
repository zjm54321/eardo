use crate::api::{list_voice_metadata, VoiceMetaInfo};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::{Deserialize, Serialize};

// 定义 metadata JSON 的结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VoiceFilterMetadata {
    #[serde(default)]
    description: String,
    #[serde(default)]
    base_model_id: String,
    #[serde(default)]
    pitch: f64,
    #[serde(default = "default_speed")]
    speed: f64,
    #[serde(default)]
    volume: f64,
    #[serde(default)]
    emotion: String,
    #[serde(default)]
    usage_count: i32,
    #[serde(default)]
    is_public: bool,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    author: String,
    #[serde(default)]
    is_official: bool,
}

fn default_speed() -> f64 {
    1.0
}

// 用于显示的滤镜结构
#[derive(Debug, Clone)]
struct DisplayFilter {
    id: String,
    name: String,
    description: String,
    base_model_id: String,
    pitch: f64,
    speed: f64,
    volume: f64,
    emotion: String,
    usage_count: i32,
    tags: Vec<String>,
    author: String,
    is_official: bool,
}

impl DisplayFilter {
    fn from_voice_meta(meta: VoiceMetaInfo) -> Self {
        let metadata: VoiceFilterMetadata =
            serde_json::from_str(&meta.metadata).unwrap_or_else(|_| VoiceFilterMetadata {
                description: String::new(),
                base_model_id: String::new(),
                pitch: 0.0,
                speed: 1.0,
                volume: 1.0,
                emotion: "normal".to_string(),
                usage_count: 0,
                is_public: true,
                tags: vec![],
                author: "未知".to_string(),
                is_official: false,
            });

        DisplayFilter {
            id: meta.id,
            name: meta.name,
            description: metadata.description,
            base_model_id: metadata.base_model_id,
            pitch: metadata.pitch,
            speed: metadata.speed,
            volume: metadata.volume,
            emotion: metadata.emotion,
            usage_count: metadata.usage_count,
            tags: metadata.tags,
            author: metadata.author,
            is_official: metadata.is_official,
        }
    }
}

#[component]
pub fn VoiceFilterPage() -> impl IntoView {
    // 获取数据资源
    let filters_resource = Resource::new(|| (), |_| list_voice_metadata());

    // 导航 hook
    let navigate = use_navigate();

    // 搜索词
    let (search, set_search) = signal(String::new());

    // 处理“使用滤镜”点击
    let apply_filter = move |filter: DisplayFilter| {
        let url = format!(
            "/home?voice_id={}&pitch={}&speed={}&emotion={}",
            filter.base_model_id, filter.pitch, filter.speed, filter.emotion
        );
        navigate(&url, Default::default());
    };

    view! {
        <div class="min-h-screen bg-base-100 pb-20 pt-8">
            <div class="container mx-auto px-4 max-w-6xl">

                // --- 1. 顶部标题 & 搜索 ---
                <section class="text-center mb-12">
                    <h2 class="text-3xl font-bold mb-4 text-dark flex items-center justify-center">
                        <i class="fa-solid fa-wand-magic-sparkles text-secondary mr-3"></i>
                        "声音滤镜库"
                    </h2>
                    <p class="text-gray-500 mb-8">
                        "发现并使用各种声音滤镜，一键应用到你的作品中"
                    </p>

                    <div class="relative max-w-xl mx-auto">
                        <input
                            type="text"
                            placeholder="搜索滤镜（名称、作者、标签、描述）"
                            class="w-full pl-12 pr-10 py-3 rounded-full border border-gray-200 bg-white focus:outline-none focus:ring-2 focus:ring-primary/50 shadow-sm"
                            prop:value=move || search.get()
                            on:input=move |ev| set_search.set(event_target_value(&ev))
                        />
                        <i class="fa-solid fa-magnifying-glass absolute left-5 top-1/2 transform -translate-y-1/2 text-gray-400"></i>
                        <Show when=move || !search.get().is_empty()>
                            <button
                                class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 px-2"
                                on:click=move |_| set_search.set(String::new())
                                title="清除"
                            >
                                <i class="fa-solid fa-xmark"></i>
                            </button>
                        </Show>
                    </div>
                </section>

                <Suspense fallback=move || {
                    view! { <div class="text-center py-10">"加载滤镜中..."</div> }
                }>
                    {move || {
                        match filters_resource.get() {
                            Some(Ok(meta_list)) => {
                                let all_filters: Vec<DisplayFilter> = meta_list
                                    .into_iter()
                                    .map(DisplayFilter::from_voice_meta)
                                    .collect();
                                let query = search.get().trim().to_lowercase();
                                let filtered: Vec<DisplayFilter> = if query.is_empty() {
                                    all_filters.clone()
                                } else {
                                    all_filters
                                        .into_iter()
                                        .filter(|f| {
                                            let name = f.name.to_lowercase();
                                            let author = f.author.to_lowercase();
                                            let desc = f.description.to_lowercase();
                                            let tag_hit = f
                                                .tags
                                                .iter()
                                                .any(|t| t.to_lowercase().contains(&query));
                                            name.contains(&query) || author.contains(&query)
                                                || desc.contains(&query) || tag_hit
                                        })
                                        .collect()
                                };
                                let (official, user): (Vec<_>, Vec<_>) = filtered
                                    .into_iter()
                                    .partition(|f| f.is_official);
                                let apply_filter_1 = apply_filter.clone();
                                let apply_filter_2 = apply_filter.clone();

                                // 根据搜索词过滤

                                // 显式克隆闭包传递给组件

                                view! {
                                    <div class="space-y-16">
                                        <FilterSection
                                            title="官方推荐滤镜"
                                            icon="fa-star"
                                            icon_color="text-primary"
                                            filters=official
                                            on_apply=apply_filter_1
                                        />

                                        <FilterSection
                                            title="用户分享滤镜"
                                            icon="fa-users"
                                            icon_color="text-secondary"
                                            filters=user
                                            on_apply=apply_filter_2
                                        />
                                    </div>
                                }
                                    .into_any()
                            }
                            Some(Err(e)) => {
                                view! {
                                    <div class="text-red-500 text-center">
                                        "加载失败: " {e.to_string()}
                                    </div>
                                }
                                    .into_any()
                            }
                            None => {
                                view! { <div class="text-center">"加载中..."</div> }.into_any()
                            }
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

// --- 子组件：滤镜区域 ---
#[component]
fn FilterSection<F>(
    title: &'static str,
    icon: &'static str,
    icon_color: &'static str,
    filters: Vec<DisplayFilter>,
    on_apply: F, // 泛型闭包
) -> impl IntoView
where
    F: Fn(DisplayFilter) + Clone + Send + 'static,
{
    view! {
        <section>
            <h3 class="text-xl font-bold mb-6 flex items-center text-gray-800">
                <i class=format!("fa {} {} mr-2", icon, icon_color)></i>
                {title}
            </h3>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <For
                    each=move || filters.clone()
                    key=|f| f.id.clone()
                    children=move |filter| {
                        let filter_clone = filter.clone();
                        let on_apply = on_apply.clone();

                        view! {
                            <div class="bg-white rounded-xl p-6 shadow-soft hover:shadow-lg transition-all duration-300 border border-gray-100 group">
                                <div class="flex justify-between items-start mb-3">
                                    <h4 class="text-lg font-bold text-gray-800">
                                        {filter.name.clone()}
                                    </h4>
                                    <span class="text-xs px-2 py-1 rounded-full bg-gray-100 text-gray-500">
                                        {if filter.is_official {
                                            "官方".to_string()
                                        } else {
                                            filter.author.clone()
                                        }}
                                    </span>
                                </div>

                                <p class="text-sm text-gray-500 mb-4 h-10 line-clamp-2">
                                    {filter.description.clone()}
                                </p>

                                <div class="flex flex-wrap gap-2 mb-6">
                                    {filter
                                        .tags
                                        .iter()
                                        .map(|tag| {
                                            view! {
                                                <span class="text-xs px-2 py-1 rounded bg-gray-50 text-gray-600 border border-gray-200">
                                                    {tag.clone()}
                                                </span>
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </div>

                                <div class="flex items-center justify-between mt-auto">
                                    <div class="text-xs text-gray-400 space-x-2">
                                        <span>
                                            <i class="fa-solid fa-signal mr-1"></i>
                                            {filter.pitch}
                                        </span>
                                        <span>
                                            <i class="fa-solid fa-gauge mr-1"></i>
                                            {filter.speed}
                                            "x"
                                        </span>
                                    </div>

                                    <button
                                        class="bg-primary/10 hover:bg-primary text-primary hover:text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center"
                                        on:click=move |_| {
                                            on_apply(filter_clone.clone());
                                        }
                                    >
                                        <i class="fa-solid fa-check mr-2"></i>
                                        "使用"
                                    </button>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </section>
    }
}
