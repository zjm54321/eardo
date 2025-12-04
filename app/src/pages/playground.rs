use crate::api::{get_featured_works, get_latest_works};
use crate::data::VoiceWork;
use leptos::prelude::*;

#[component]
pub fn Playground() -> impl IntoView {
    // --- 状态管理 ---
    // 1. 轮播图状态
    let (current_slide, set_current_slide) = signal(0);
    // 资源：获取精选作品
    let featured_resource = Resource::new(|| (), |_| get_featured_works());

    // 2. 列表作品状态
    let (page, set_page) = signal(1);
    // 资源：获取列表作品 (依赖 page)
    // 注意：这里我们简化处理，实际无限加载通常需要将新数据追加到旧数据列表
    // 但为了 Resource 的简单性，我们先只展示当前页，或者使用 create_local_resource + action 来手动追加
    // 这里演示使用 Action 来手动加载更多并追加到信号中
    let (works_list, set_works_list) = signal(Vec::<VoiceWork>::new());

    // Action: 加载更多
    let load_more_action = Action::new(move |_| {
        let current_p = page.get() + 1;
        async move {
            match get_latest_works(current_p, 6).await {
                Ok(new_works) => {
                    if !new_works.is_empty() {
                        set_works_list.update(|list| list.extend(new_works));
                        set_page.set(current_p);
                    }
                }
                Err(e) => leptos::logging::error!("加载更多失败: {}", e),
            }
        }
    });

    // --- 交互逻辑 ---
    let next_slide = move |total: usize| {
        set_current_slide.update(|i| *i = (*i + 1) % total);
    };

    let prev_slide = move |total: usize| {
        set_current_slide.update(|i| *i = (*i + total - 1) % total);
    };

    view! {
        <div class="min-h-screen bg-base-100 pb-20 pt-8">
            <div class="container mx-auto px-4 max-w-6xl">

                // --- 1. 顶部标题 ---
                <section class="text-center mb-12">
                    <h2 class="text-[clamp(1.8rem,4vw,2.5rem)] font-bold mb-4 text-shadow flex items-center justify-center">
                        <i class="fa fa-users text-secondary mr-2"></i>
                        "声音广场"
                    </h2>
                    <p class="text-gray-600 max-w-2xl mx-auto">"分享您的创意声音作品，聆听他人的声音世界"</p>
                </section>

                // --- 2. 精选作品轮播 ---
                <section class="mb-16">
                    <Suspense fallback=|| view! { <div class="text-center py-10">"加载精选作品..."</div> }>
                        {move || {
                            featured_resource.get().map(|res| match res {
                                Ok(featured) if !featured.is_empty() => {
                                    let total = featured.len();
                                    view! {
                                        <div class="relative">
                                            // 标题栏 + 控制按钮
                                            <div class="flex justify-between items-center mb-6">
                                                <h3 class="text-xl font-semibold">"精选作品"</h3>
                                                <div class="flex space-x-2">
                                                    <button
                                                        class="w-10 h-10 rounded-full border border-gray-300 flex items-center justify-center hover:border-primary hover:text-primary transition-all duration-300"
                                                        on:click=move |_| prev_slide(total)
                                                    >
                                                        <i class="fa fa-chevron-left"></i>
                                                    </button>
                                                    <button
                                                        class="w-10 h-10 rounded-full border border-gray-300 flex items-center justify-center hover:border-primary hover:text-primary transition-all duration-300"
                                                        on:click=move |_| next_slide(total)
                                                    >
                                                        <i class="fa fa-chevron-right"></i>
                                                    </button>
                                                    </div>
                                            </div>

                                            // 轮播内容
                                            <div class="relative overflow-hidden rounded-2xl min-h-[300px]">
                                                {featured.into_iter().enumerate().map(|(idx, work)| {
                                                    view! {
                                                        <div
                                                            class="transition-all duration-500 ease-in-out"
                                                            class:hidden=move || current_slide.get() != idx
                                                            class:block=move || current_slide.get() == idx
                                                            class:animate-fade-in=move || current_slide.get() == idx
                                                        >
                                                            <WorkCard work=work is_featured=true />
                                                        </div>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>

                                            // 指示器
                                            <div class="flex justify-center mt-6 space-x-2">
                                                {(0..total).map(|idx| {
                                                    view! {
                                                        <button
                                                            class="w-3 h-3 rounded-full transition-colors duration-300"
                                                            class:bg-primary=move || current_slide.get() == idx
                                                            class:bg-gray-300=move || current_slide.get() != idx
                                                            on:click=move |_| set_current_slide.set(idx)
                                                        />
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                    }.into_any()
                                }
                                _ => view! { <div class="text-gray-500 text-center">"暂无精选作品"</div> }.into_any()
                            })
                        }}
                    </Suspense>
                </section>

                // --- 3. 最新作品列表 ---
                <section>
                    <h3 class="text-xl font-semibold mb-6">"最新作品"</h3>

                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        <For
                            each=move || works_list.get()
                            key=|w| w.id
                            children=move |work| {
                                view! { <WorkCard work=work is_featured=false /> }
                            }
                        />
                    </div>

                    // 加载更多按钮
                    <div class="text-center mt-10">
                        <button
                            class="border border-secondary text-secondary hover:bg-secondary hover:text-white py-3 px-8 rounded-lg transition-all duration-300 disabled:opacity-50 disabled:cursor-not-allowed"
                            on:click=move |_| { load_more_action.dispatch(()); }
                            disabled=move || load_more_action.pending().get()
                        >
                            {move || if load_more_action.pending().get() {
                                "加载中..."
                            } else {
                                "加载更多作品"
                            }}
                        </button>
                    </div>
                </section>

            </div>
        </div>
    }
}

// --- 子组件：作品卡片 ---
#[component]
fn WorkCard(work: VoiceWork, is_featured: bool) -> impl IntoView {
    // 简单的点赞状态 (仅前端模拟)
    let (liked, set_liked) = signal(false);
    let (like_count, set_like_count) = signal(work.likes);

    let toggle_like = move |_| {
        set_liked.update(|v| *v = !*v);
        set_like_count.update(|c| if liked.get() { *c += 1 } else { *c -= 1 });
    };

    view! {
        <div class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover h-full flex flex-col"
             class:max-w-2xl=is_featured
             class:mx-auto=is_featured>

            // 用户信息
            <div class="flex items-center mb-4">
                <img src=work.avatar alt="Avatar" class="w-12 h-12 rounded-full mr-3 object-cover" />
                <div>
                    <div class="font-medium text-gray-800">{work.author}</div>
                    <div class="text-xs text-gray-500">{work.time}</div>
                </div>
            </div>

            // 内容
            <h3 class="font-semibold mb-3 text-lg text-dark">{work.title}</h3>
            <p class="text-sm text-gray-600 mb-4 line-clamp-3">{work.description}</p>

            // 音频播放器
            <div class="mb-4 bg-gray-50 rounded-lg p-2">
                <div class="flex items-center justify-between mb-2 px-1">
                    <p class="text-xs text-gray-500">"作品音频"</p>
                    <span class="text-xs text-primary cursor-pointer hover:underline">
                        <i class="fa fa-refresh mr-1"></i> "替换"
                    </span>
                </div>
                <audio controls class="w-full h-8" src=work.audio_url>
                    "您的浏览器不支持音频播放"
                </audio>
            </div>

            // 底部操作栏 (mt-auto 保证对齐底部)
            <div class="flex justify-between items-center text-sm mt-auto pt-2">
                <div class="flex items-center text-gray-500 space-x-4">
                    <button
                        class="flex items-center transition-colors duration-200 group"
                        class:text-accent=move || liked.get()
                        on:click=toggle_like
                    >
                        <i class="mr-1" class:fa-heart=move || liked.get() class:fa-heart-o=move || !liked.get()></i>
                        <span>{move || like_count.get()}</span>
                    </button>
                    <button class="flex items-center hover:text-primary transition-colors duration-200">
                        <i class="fa fa-comment-o mr-1"></i>
                        <span>{work.comments}</span>
                    </button>
                </div>
                <span class="text-secondary bg-secondary/10 px-2 py-1 rounded-full text-xs">
                    {work.voice_type}
                </span>
            </div>
        </div>
    }
}
