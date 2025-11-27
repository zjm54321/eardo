use crate::api::get_voice_filters;
use crate::data::{self, VoiceFilter};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

#[component]
pub fn VoiceFilterPage() -> impl IntoView {
    // 获取数据资源
    let filters_resource = Resource::new(|| (), |_| get_voice_filters());

    // 导航 hook
    let navigate = use_navigate();

    // 处理“使用滤镜”点击
    // 这个闭包捕获了 navigate，navigate 实现了 Clone，所以这个闭包也实现了 Clone
    // 但它没有实现 Copy，所以我们需要在传递时注意
    let apply_filter = move |filter: VoiceFilter| {
        let url = format!(
            "/?voice_id={}&pitch={}&speed={}&emotion={}",
            filter.voice_data.voice_id,
            filter.voice_data.voice_params.pitch,
            filter.voice_data.voice_params.speed,
            filter.voice_data.voice_params.emotion
        );
        navigate(&url, Default::default());
    };

    view! {
        <div class="min-h-screen bg-base-100 pb-20 pt-8">
            <div class="container mx-auto px-4 max-w-6xl">

                // --- 1. 顶部标题 & 搜索占位 ---
                <section class="text-center mb-12">
                    <h2 class="text-3xl font-bold mb-4 text-dark flex items-center justify-center">
                        <i class="fa fa-magic text-secondary mr-3"></i>
                        "声音滤镜库"
                    </h2>
                    <p class="text-gray-500 mb-8">"发现并使用各种声音滤镜，一键应用到你的作品中"</p>

                    <div class="relative max-w-xl mx-auto opacity-60 hover:opacity-100 transition-opacity">
                        <input
                            type="text"
                            placeholder="搜索滤镜 (功能开发中...)"
                            disabled
                            class="w-full pl-12 pr-4 py-3 rounded-full border border-gray-200 bg-white focus:outline-none focus:ring-2 focus:ring-primary/50 shadow-sm cursor-not-allowed"
                        />
                        <i class="fa fa-search absolute left-5 top-1/2 transform -translate-y-1/2 text-gray-400"></i>
                    </div>
                </section>

                <Suspense fallback=move || view! { <div class="text-center py-10">"加载滤镜中..."</div> }>
                    {move || {
                        match filters_resource.get() {
                            Some(Ok(all_filters)) => {
                                let (official, user): (Vec<_>, Vec<_>) = all_filters.into_iter()
                                    .partition(|f| f.state == data::DisplayState::Official);

                                // 显式克隆闭包传递给组件
                                let apply_filter_1 = apply_filter.clone();
                                let apply_filter_2 = apply_filter.clone();

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
                                }.into_any()
                            },
                            Some(Err(e)) => view! {
                                <div class="text-red-500 text-center">"加载失败: " {e.to_string()}</div>
                            }.into_any(),
                            None => view! { <div class="text-center">"加载中..."</div> }.into_any()
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
    filters: Vec<VoiceFilter>,
    on_apply: F, // 泛型闭包
) -> impl IntoView
where
    // 关键修改：移除了 Copy 约束
    // 我们保留 Clone 和 Send (For 组件通常需要 Send)
    F: Fn(VoiceFilter) + Clone + Send + 'static,
{
    view! {
        <section>
            <h3 class="text-xl font-bold mb-6 flex items-center text-gray-800">
                <i class={format!("fa {} {} mr-2", icon, icon_color)}></i>
                {title}
            </h3>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <For
                    each=move || filters.clone()
                    key=|f| f.id.clone()
                    children=move |filter| {
                        let filter_clone = filter.clone();

                        // 关键修改：因为 F 不是 Copy 的，我们必须在这里显式 clone 它
                        // children 闭包是 Fn，所以它借用了 on_apply，我们可以调用 .clone() 得到一个新的所有权闭包
                        let on_apply = on_apply.clone();

                        view! {
                            <div class="bg-white rounded-xl p-6 shadow-soft hover:shadow-lg transition-all duration-300 border border-gray-100 group">
                                <div class="flex justify-between items-start mb-3">
                                    <h4 class="text-lg font-bold text-gray-800">{filter.name.clone()}</h4>
                                    <span class="text-xs px-2 py-1 rounded-full bg-gray-100 text-gray-500">
                                        {if filter.state == data::DisplayState::Official { "官方".to_string() } else { filter.author.clone() }}
                                    </span>
                                </div>

                                <p class="text-sm text-gray-500 mb-4 h-10 line-clamp-2">
                                    {filter.desc.clone()}
                                </p>

                                <div class="flex flex-wrap gap-2 mb-6">
                                    {filter.tags.iter().map(|tag| view! {
                                        <span class="text-xs px-2 py-1 rounded bg-gray-50 text-gray-600 border border-gray-200">
                                            {tag.clone()}
                                        </span>
                                    }).collect::<Vec<_>>()}
                                </div>

                                <div class="flex items-center justify-between mt-auto">
                                    <div class="text-xs text-gray-400 space-x-2">
                                        <span><i class="fa fa-signal mr-1"></i>{filter.voice_data.voice_params.pitch}</span>
                                        <span><i class="fa fa-tachometer mr-1"></i>{filter.voice_data.voice_params.speed}"x"</span>
                                    </div>

                                    <button
                                        class="bg-primary/10 hover:bg-primary text-primary hover:text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center"
                                        on:click=move |_| {
                                            // 这里使用的是上面 clone 进来的 on_apply
                                            on_apply(filter_clone.clone());
                                        }
                                    >
                                        <i class="fa fa-check mr-2"></i>
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
