use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HelpPage() -> impl IntoView {
    let faq_items = vec![
        (
            "如何获取自己的声音？",
            vec![
                "1. 在首页输入想要转换的文本",
                "2. 选择一个喜欢的声线（如小小姐、龙小小等）",
                "3. 调整音高、速度等参数",
                "4. 点击生成按钮，系统即可生成对应的音频",
                "5. 生成后可以直接播放或分享到社区",
            ],
        ),
        (
            "怎样调整音频？",
            vec![
                "声音调整主要包括三个方面：",
                "",
                "• 音高(Pitch)：调整语音的高低，范围0.5-2.5，数值越大音高越高",
                "• 速度(Speed)：调整语速快慢，范围0.5-2.5，数值越大说话越快",
                "• 音色(Emotion)：选择不同的情感表达（开心、生气、冷静等）",
                "",
                "在参数调节区域拖动滑块或直接输入数值即可实时预览效果",
            ],
        ),
        (
            "声音可以用来做什么？",
            vec![
                "耳朵提供的音频生成功能可以用于多种用途：",
                "",
                "• 内容创作：为视频、播客配音",
                "• 学习辅助：将文字转换为语音，加深记忆",
                "• 无障碍服务：为视力受损的用户提供文本朗读",
                "• 品牌推广：制作个性化的语音广告或欢迎语",
                "• 社交分享：生成创意音频作品分享给朋友",
                "• 应用集成：为自己的应用或网站添加语音功能",
            ],
        ),
        (
            "若音频出现问题，如何解决？",
            vec![
                "常见问题解决方案：",
                "",
                "• 音频无法生成：检查网络连接，确保文本不为空",
                "• 生成速度慢：这可能是服务器繁忙，请耐心等待",
                "• 音频质量差：尝试调整参数，建议音高和速度在0.8-1.2之间",
                "• 播放出现卡顿：刷新页面或清空浏览器缓存",
                "• 仍有问题：可以尝试更换声线或重新输入文本",
                "",
                "如果问题仍未解决，欢迎在社区反馈或联系技术支持团队",
            ],
        ),
    ];

    view! {
        <div class="min-h-screen bg-base-100 pb-12">
            <div class="container mx-auto px-4 py-8 md:py-12 max-w-4xl">
                // 页面头部
                <section class="text-center mb-12">
                    <h1 class="text-[clamp(2rem,5vw,3rem)] font-bold mb-4 text-shadow text-dark">
                        "帮助中心"
                    </h1>
                    <p class="text-gray-600 max-w-2xl mx-auto text-lg">
                        "遇到问题？这里有你需要的所有答案。浏览常见问题或直接搜索你感兴趣的内容。"
                    </p>
                </section>

                // FAQ 列表
                <div class="space-y-6">
                    {faq_items
                        .into_iter()
                        .map(|(question, answers)| {
                            let show_detail = RwSignal::new(false);

                            view! {
                                <div class="bg-white rounded-lg border border-gray-200 shadow-sm hover:shadow-md transition-shadow duration-300 overflow-hidden">
                                    // 问题头部 - 可点击展开
                                    <button
                                        class="w-full px-6 py-4 md:px-8 md:py-5 flex items-start justify-between hover:bg-gray-50 transition-colors duration-200 text-left"
                                        on:click=move |_| show_detail.set(!show_detail.get())
                                    >
                                        <h3 class="text-lg md:text-xl font-semibold text-gray-800 pr-4 flex-1">
                                            {question}
                                        </h3>
                                        <i class=move || {
                                            if show_detail.get() {
                                                "fa-solid fa-chevron-up text-primary flex-shrink-0"
                                            } else {
                                                "fa-solid fa-chevron-down text-gray-400 flex-shrink-0"
                                            }
                                        }></i>
                                    </button>

                                    // 答案内容 - 展开/收起
                                    <div
                                        class=move || {
                                            if show_detail.get() { "max-h-96" } else { "max-h-0" }
                                        }
                                        style="overflow: hidden; transition: max-height 0.3s ease-in-out;"
                                    >
                                        <div class="px-6 py-6 md:px-8 md:py-6 bg-gray-50 border-t border-gray-200">
                                            {answers
                                                .into_iter()
                                                .map(|answer| {
                                                    view! {
                                                        <p class="text-gray-700 leading-relaxed mb-3 text-base">
                                                            {answer}
                                                        </p>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>

                // 底部提示
                <div class="mt-12 bg-gradient-to-r from-primary/5 to-secondary/5 rounded-lg border border-primary/20 p-8 text-center">
                    <h3 class="text-xl font-semibold text-gray-800 mb-3">
                        "还有其他问题？"
                    </h3>
                    <p class="text-gray-600 mb-6">
                        "如果你没有找到答案，欢迎在社区提问或联系我们的技术支持团队。"
                    </p>
                    <div class="flex gap-4 justify-center flex-wrap">
                        <A
                            href="/voice"
                            attr:class="px-6 py-2 bg-primary hover:bg-primary/90 text-white rounded-lg transition-colors duration-300 font-medium"
                        >
                            "返回声音广场"
                        </A>
                        <A
                            href="/home"
                            attr:class="px-6 py-2 border border-primary text-primary hover:bg-primary/5 rounded-lg transition-colors duration-300 font-medium"
                        >
                            "返回首页"
                        </A>
                    </div>
                </div>
            </div>
        </div>
    }
}
