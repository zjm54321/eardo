use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn WorkbenchHomePage() -> impl IntoView {
    view! {
        <div class="min-h-screen pb-12 bg-gray-50/30">
            <div class="container mx-auto max-w-[1280px] px-4 py-8 md:py-12">
                // Header Section
                <div class="mb-10">
                    <h1 class="text-3xl md:text-4xl font-bold text-gray-900 flex items-center gap-2">
                        "创造你的声音，表达你的世界" <span class="text-2xl">"✨"</span>
                    </h1>
                    <div class="mt-4 flex flex-col md:flex-row md:items-center gap-4 text-gray-500">
                        <p class="text-base">"从灵感到声音，只需简单几步"</p>
                        <div class="hidden md:flex items-center gap-2 text-sm">
                            <span class="flex items-center gap-1.5 bg-white px-3 py-1 rounded-full border border-emerald-200 shadow-sm text-emerald-700">
                                <span class="w-5 h-5 rounded-full bg-emerald-500 text-white flex items-center justify-center text-xs font-bold">"1"</span>
                                "选择场景"
                            </span>
                            <i class="fa-solid fa-arrow-right text-gray-300 text-xs"></i>
                            <span class="flex items-center gap-1.5 bg-white px-3 py-1 rounded-full border border-gray-200 shadow-sm text-gray-600">
                                <span class="w-5 h-5 rounded-full bg-gray-100 text-gray-400 flex items-center justify-center text-xs font-bold">"2"</span>
                                "输入内容"
                            </span>
                            <i class="fa-solid fa-arrow-right text-gray-300 text-xs"></i>
                            <span class="flex items-center gap-1.5 bg-white px-3 py-1 rounded-full border border-gray-200 shadow-sm text-gray-600">
                                <span class="w-5 h-5 rounded-full bg-gray-100 text-gray-400 flex items-center justify-center text-xs font-bold">"3"</span>
                                "选择滤镜"
                            </span>
                            <i class="fa-solid fa-arrow-right text-gray-300 text-xs"></i>
                            <span class="flex items-center gap-1.5 bg-white px-3 py-1 rounded-full border border-gray-200 shadow-sm text-gray-600">
                                <span class="w-5 h-5 rounded-full bg-gray-100 text-gray-400 flex items-center justify-center text-xs font-bold">"4"</span>
                                "生成声音"
                            </span>
                        </div>
                    </div>
                </div>

                <div class="grid lg:grid-cols-[1fr_300px] gap-8 items-start">
                    // Main Content - Scenarios
                    <div class="space-y-6">
                        <div class="relative">
                            <div class="absolute -top-3.5 left-6 z-10">
                                <span class="inline-flex items-center gap-1.5 rounded-full bg-[#FFF8E7] px-3 py-1 text-sm font-semibold text-[#D97706] border border-[#FDE68A] shadow-sm">
                                    <span class="w-5 h-5 rounded-full bg-[#F59E0B] text-white flex items-center justify-center text-xs">"1"</span>
                                    "场景选择入口（首页首屏）"
                                </span>
                            </div>

                            <div class="rounded-3xl border border-[#FDE68A] bg-white p-6 pt-10 shadow-sm">
                                <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
                                    // Scenario Card 1: 汇报 / 演讲
                                    <A href="/scene-create?scene=presentation" attr:class="group relative overflow-hidden rounded-2xl border border-emerald-100 bg-white p-5 transition-all hover:shadow-md hover:border-emerald-300 hover:-translate-y-0.5">
                                        <div class="flex flex-col gap-3">
                                            <div class="flex items-center gap-3">
                                                <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-emerald-50 text-emerald-500 transition-transform group-hover:scale-110">
                                                    <i class="fa-solid fa-chalkboard-user text-2xl"></i>
                                                </div>
                                                <h3 class="font-bold text-gray-900 text-lg">"汇报 / 演讲"</h3>
                                            </div>
                                            <div class="space-y-1.5 pl-1">
                                                <p class="text-sm text-gray-500">"课程汇报、答辩演讲"</p>
                                                <p class="text-sm text-gray-500">"工作汇报、项目展示"</p>
                                            </div>
                                        </div>
                                    </A>

                                    // Scenario Card 2: 内容创作
                                    <A href="/scene-create?scene=content" attr:class="group relative overflow-hidden rounded-2xl border border-blue-100 bg-white p-5 transition-all hover:shadow-md hover:border-blue-300 hover:-translate-y-0.5">
                                        <div class="flex flex-col gap-3">
                                            <div class="flex items-center gap-3">
                                                <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-blue-50 text-blue-500 transition-transform group-hover:scale-110">
                                                    <i class="fa-solid fa-photo-film text-2xl"></i>
                                                </div>
                                                <h3 class="font-bold text-gray-900 text-lg">"内容创作"</h3>
                                            </div>
                                            <div class="space-y-1.5 pl-1">
                                                <p class="text-sm text-gray-500">"短视频配音、影视解说"</p>
                                                <p class="text-sm text-gray-500">"播客制作、人设配音"</p>
                                            </div>
                                        </div>
                                    </A>

                                    // Scenario Card 3: 娱乐 / 配音
                                    <A href="/scene-create?scene=dubbing" attr:class="group relative overflow-hidden rounded-2xl border border-purple-100 bg-white p-5 transition-all hover:shadow-md hover:border-purple-300 hover:-translate-y-0.5">
                                        <div class="flex flex-col gap-3">
                                            <div class="flex items-center gap-3">
                                                <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-purple-50 text-purple-500 transition-transform group-hover:scale-110">
                                                    <i class="fa-solid fa-gamepad text-2xl"></i>
                                                </div>
                                                <h3 class="font-bold text-gray-900 text-lg">"娱乐 / 配音"</h3>
                                            </div>
                                            <div class="space-y-1.5 pl-1">
                                                <p class="text-sm text-gray-500">"角色扮演、游戏配音"</p>
                                                <p class="text-sm text-gray-500">"趣味变声、模仿秀"</p>
                                            </div>
                                        </div>
                                    </A>

                                    // Scenario Card 4: 表达提升
                                    <A href="/scene-create?scene=expression" attr:class="group relative overflow-hidden rounded-2xl border border-amber-100 bg-white p-5 transition-all hover:shadow-md hover:border-amber-300 hover:-translate-y-0.5">
                                        <div class="flex flex-col gap-3">
                                            <div class="flex items-center gap-3">
                                                <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-amber-50 text-amber-500 transition-transform group-hover:scale-110">
                                                    <i class="fa-solid fa-microphone-lines text-2xl"></i>
                                                </div>
                                                <h3 class="font-bold text-gray-900 text-lg">"表达提升"</h3>
                                            </div>
                                            <div class="space-y-1.5 pl-1">
                                                <p class="text-sm text-gray-500">"演讲练习、口语训练"</p>
                                                <p class="text-sm text-gray-500">"普通话练习、情绪表达"</p>
                                            </div>
                                        </div>
                                    </A>

                                    // Scenario Card 5: 专业表达
                                    <A href="/scene-create?scene=professional" attr:class="group relative overflow-hidden rounded-2xl border border-teal-100 bg-white p-5 transition-all hover:shadow-md hover:border-teal-300 hover:-translate-y-0.5">
                                        <div class="flex flex-col gap-3">
                                            <div class="flex items-center gap-3">
                                                <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-xl bg-teal-50 text-teal-500 transition-transform group-hover:scale-110">
                                                    <i class="fa-solid fa-building-columns text-2xl"></i>
                                                </div>
                                                <h3 class="font-bold text-gray-900 text-lg">"专业表达"</h3>
                                            </div>
                                            <div class="space-y-1.5 pl-1">
                                                <p class="text-sm text-gray-500">"航运讲解、工程汇报"</p>
                                                <p class="text-sm text-gray-500">"金融分析、学术表达"</p>
                                            </div>
                                        </div>
                                    </A>

                                    // Scenario Card 6: 自定义/探索
                                    <A href="/scene-create?scene=custom" attr:class="group relative overflow-hidden rounded-2xl border border-dashed border-gray-200 bg-gray-50/50 p-5 transition-all hover:shadow-md hover:border-gray-400 hover:bg-gray-50 flex flex-col items-center justify-center text-center min-h-[140px]">
                                        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-white border border-gray-200 text-gray-400 transition-transform group-hover:scale-110 mb-2 shadow-sm">
                                            <i class="fa-solid fa-plus"></i>
                                        </div>
                                        <h3 class="font-bold text-gray-600">"自定义场景"</h3>
                                        <p class="text-xs text-gray-400 mt-1">"从空白场景开始"</p>
                                    </A>
                                </div>
                            </div>
                        </div>

                    </div>

                    // Sidebar - Flow & Roles
                    <div class="space-y-6">
                        // Flow Description Box
                        <div class="rounded-3xl border border-[#FDE68A] bg-[#FFFBEB] p-6 shadow-sm">
                            <h3 class="font-bold text-gray-900 mb-5">"整体流程说明"</h3>
                            <div class="space-y-5 relative before:absolute before:inset-0 before:ml-[11px] before:-translate-x-px before:h-full before:w-0.5 before:bg-[#FDE68A]">
                                <div class="relative flex items-start gap-3">
                                    <div class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-[#FCD34D] text-[#92400E] text-xs font-bold ring-4 ring-[#FFFBEB] z-10">"1"</div>
                                    <div class="pt-0.5">
                                        <h4 class="font-semibold text-gray-900 text-sm">"选择场景（首页首屏）"</h4>
                                        <p class="mt-0.5 text-xs text-gray-500">"快速定位你的使用目的"</p>
                                    </div>
                                </div>
                                <div class="relative flex items-start gap-3">
                                    <div class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-[#FDE68A] text-[#92400E] text-xs font-bold ring-4 ring-[#FFFBEB] z-10">"2"</div>
                                    <div class="pt-0.5">
                                        <h4 class="font-semibold text-gray-900 text-sm">"进入创作页（表达助手）"</h4>
                                        <p class="mt-0.5 text-xs text-gray-500">"输入内容 → AI 优化表达 → 选择滤镜 → 生成声音"</p>
                                    </div>
                                </div>
                                <div class="relative flex items-start gap-3">
                                    <div class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-[#FEF08A] text-[#92400E] text-xs font-bold ring-4 ring-[#FFFBEB] z-10">"3"</div>
                                    <div class="pt-0.5">
                                        <h4 class="font-semibold text-gray-900 text-sm">"发布到广场（可选）"</h4>
                                        <p class="mt-0.5 text-xs text-gray-500">"分享你的作品到社区"</p>
                                    </div>
                                </div>
                                <div class="relative flex items-start gap-3">
                                    <div class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-[#FEF9C3] text-[#92400E] text-xs font-bold ring-4 ring-[#FFFBEB] z-10">"4"</div>
                                    <div class="pt-0.5">
                                        <h4 class="font-semibold text-gray-900 text-sm">"他人使用 / 分享滤镜"</h4>
                                        <p class="mt-0.5 text-xs text-gray-500">"在“声音滤镜分享”页发现并使用优质滤镜"</p>
                                    </div>
                                </div>
                            </div>
                        </div>

                        // Role Distinction Box
                        <div class="rounded-3xl border border-gray-200 bg-white p-6 shadow-sm">
                            <h3 class="font-bold text-gray-900 mb-4">"角色区别"</h3>

                            <div class="space-y-4">
                                <div class="rounded-2xl bg-emerald-50/50 p-4 border border-emerald-100">
                                    <div class="flex items-center gap-3 mb-2">
                                        <div class="w-8 h-8 rounded-full bg-emerald-100 flex items-center justify-center text-emerald-600">
                                            <i class="fa-solid fa-user text-sm"></i>
                                        </div>
                                        <h4 class="font-bold text-emerald-900 text-sm">"普通用户"</h4>
                                    </div>
                                    <ul class="text-xs text-gray-600 space-y-1.5 ml-11 list-disc pl-1 marker:text-emerald-400">
                                        <li>"选择场景"</li>
                                        <li>"选择滤镜"</li>
                                        <li>"生成使用"</li>
                                    </ul>
                                    <div class="mt-3 ml-11">
                                        <span class="inline-block bg-emerald-100/50 text-emerald-700 text-[10px] px-2 py-0.5 rounded-full border border-emerald-200/50">"操作简单，快速出声音"</span>
                                    </div>
                                </div>

                                <div class="rounded-2xl bg-amber-50/50 p-4 border border-amber-100">
                                    <div class="flex items-center gap-3 mb-2">
                                        <div class="w-8 h-8 rounded-full bg-amber-100 flex items-center justify-center text-amber-600">
                                            <i class="fa-solid fa-crown text-sm"></i>
                                        </div>
                                        <h4 class="font-bold text-amber-900 text-sm">"创作者/专业用户"</h4>
                                    </div>
                                    <ul class="text-xs text-gray-600 space-y-1.5 ml-11 list-disc pl-1 marker:text-amber-400">
                                        <li>"创建/编辑滤镜"</li>
                                        <li>"调整底层声线与参数"</li>
                                        <li>"发布分享"</li>
                                    </ul>
                                    <div class="mt-3 ml-11">
                                        <span class="inline-block bg-amber-100/50 text-amber-700 text-[10px] px-2 py-0.5 rounded-full border border-amber-200/50">"深度创作，沉淀声线资产"</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
