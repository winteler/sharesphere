use leptos::either::Either;
use leptos::prelude::*;
use sharesphere_core::sidebar::HomeSidebar;
use sharesphere_utils::routes::{CONTENT_POLICY_ROUTE, PRIVACY_POLICY_ROUTE, RULES_ROUTE};

use sharesphere_core::state::GlobalState;
use sharesphere_utils::errors::{ErrorDisplay};
use sharesphere_utils::icons::{LoadingIcon, NsfwIcon, SpoilerIcon};
use sharesphere_utils::widget::ContentBody;

#[component]
pub fn AboutShareSphere() -> impl IntoView {
    view! {
        <div class="w-full overflow-y-auto">
            <div class="flex flex-col gap-4 w-4/5 2xl:w-1/2 3xl:w-2/5 mx-auto py-4">
                <h1 class="text-3xl font-bold text-center">"About ShareSphere"</h1>
                <p class="text-justify">
                    "ShareSphere is the place to exchange with other people about your hobbies, art, news, jokes and many more topics. \
                    ShareSphere is a non-profit, ad-free, open source website with a focus on transparency, privacy and community empowerment. \
                    ShareSphere's goal is to run by relying solely on donations and to provide a better user experience than ad-based platforms."
                </p>
                <h2 class="text-xl font-semibold">"Rules & Moderation"</h2>
                <p class="text-justify">
                    "ShareSphere aims to be a place for positive and constructive exchanges. In order to make it so, a"
                    <a href=RULES_ROUTE>"base set of rules"</a>
                    " needs to be respected site-wide. Communities can define additional rules to define what content is appropriate and how users should behave."
                </p>
                <Roadmap/>
                <OriginsAndGoals/>
            </div>
        </div>
        <HomeSidebar/>
    }
}

#[component]
pub fn TermsAndConditions() -> impl IntoView {
    view! {
        <div class="w-full overflow-y-auto">
            <div class="flex flex-col gap-4 items-center w-4/5 2xl:w-1/2 3xl:w-2/5 mx-auto py-4">
                <h1 class="text-3xl font-bold text-center">"Terms and Conditions"</h1>
                <ShareSphereInfo/>
                <AcceptanceOfTerms/>
                <DescriptionOfService/>
                <UserResponsabilities/>
                <Moderation/>
                <IntellectualProperty/>
                <LimitationOfLiability/>
                <DataProtection/>
                <Amendments/>
                <GoverningLaw/>
            </div>
        </div>
        <HomeSidebar/>
    }
}

#[component]
pub fn PrivacyPolicy() -> impl IntoView {
    view! {
        <div class="w-full overflow-y-auto">
            <div class="flex flex-col gap-4 items-center w-4/5 2xl:w-1/2 3xl:w-2/5 mx-auto py-4">
                <h1 class="text-3xl font-bold text-center">"Privacy Policy"</h1>
                <ShareSphereInfo/>
                <AboutPrivacyPolicy/>
                <DataCollection/>
                <DataCollectionPurpose/>
                <LegalBasis/>
                <Cookies/>
                <DataSharing/>
                <DataStorage/>
                <UserRights/>
                <PrivacyPolicyChanges/>
            </div>
        </div>
        <HomeSidebar/>
    }
}

#[component]
pub fn ContentPolicy() -> impl IntoView {
    view! {
        <div class="w-full overflow-y-auto">
            <div class="flex flex-col gap-4 w-4/5 2xl:w-1/2 3xl:w-2/5 mx-auto py-4">
                <h1 class="text-3xl font-bold text-center">"Content Policy"</h1>
                <p class="text-justify">
                    "To ensure a good experience on ShareSphere, it is vital to exclude illegal, malicious and other problematic content, as well as properly label sensitive content.\
                    This page documents which content are forbidden or sensitive, as well as how sensitive content should be labeled."
                </p>
                <div class="flex flex-col gap-2">
                    <h2 class="text-xl font-semibold">"Banned Content"</h2>
                    <p>"The following contents are strictly prohibited on ShareSphere. It will be immediately removed and lead to a permanent ban."</p>
                    <ul class="list-disc list-inside">
                        <li>"sexual, abusive or suggestive content of minors or individuals that did not give their consent"</li>
                        <li>"human trafficking"</li>
                        <li>"paid services involving physical sexual contact"</li>
                        <li>"personal or confidential information of other individuals"</li>
                        <li>"impersonating other individuals"</li>
                        <li>"trade of stolen goods"</li>
                        <li>"falsified documents or currency"</li>
                        <li>"phishing, scams and other fraudulent schemes"</li>
                        <li>"malware or viruses"</li>
                        <li>"promotion or support of terrorism, hate crimes or any other violent ideologies"</li>
                    </ul>
                </div>
                <div class="flex flex-col gap-2">
                    <h2 class="text-xl font-semibold">"Sensitive Content"</h2>
                    <h3 class="text-lg font-semibold">"Mature Content"</h3>
                    <div class="text-justify">
                        "Content that is not suitable for minors, such as sexually explicit, graphic, violent or offensive contents must be labelled with the 'NSFW' tag: "
                        <NsfwIcon class="inline-flex"/>
                    </div>
                    <h3 class="text-lg font-semibold">"Spoiler Content"</h3>
                    <div class="text-justify">
                        "Content that could spoil information to other users, such as the content of a book or movie \
                        must keep any information out of its title and has to be labelled with the 'Spoiler' tag: "
                        <div class="h-fit w-fit px-1 py-0.5 bg-black rounded-full inline-flex relative top-1"><SpoilerIcon/></div>
                    </div>
                    <p>"The labelling rules for different content types are as follows:"</p>
                    <ul class="list-disc list-inside text-justify">
                        <li>"Any plot relevant information of books, movie, games, TV shows must always be labelled as spoiler."</li>
                        <li>"Results of competitions (sports, e-sport, games, etc.) and other live events must be labelled as spoiler in the week following the result."</li>
                    </ul>
                    <p class="text-justify">
                        "In addition, the title must make it clear spoilers will be contained in the body of the post. \
                        Spoilers in comments should be hidden using markdown formatting. Communities can also set stricter rules for spoilers."
                    </p>
                </div>
            </div>
        </div>
        <HomeSidebar/>
    }
}

#[component]
pub fn Roadmap() -> impl IntoView {
    view! {
        <h2 class="text-xl font-semibold">"Roadmap"</h2>
        <ul class="list-disc list-inside text-justify">
            <li>"Time filters"</li>
            <li>"Private messages"</li>
            <li>"Additional moderation tools"</li>
            <li>"Memberships - users can become contributors that get access to additional features"</li>
        </ul>
    }
}
#[component]
pub fn OriginsAndGoals() -> impl IntoView {
    view! {
        <h2 class="text-xl font-semibold">"Origin & Goals"</h2>
        <p class="text-justify">
            "I started thinking about ShareSphere around summer 2023 after several social media platforms I used made changes with a negative impacts on their user base. \
            I already had quite a low opinion of most social networks, thinking they were pretty terrible for the mental health of many of their users and that their \
            ad-based profit model is fundamentally incompatible with a good user experience."
        </p>
        <p class="text-justify">
            "This gave me the idea to try building a better platform, one that would non-profit, rely on donations instead of ads and would be focus on transparency. \
            Being non-profit and relying on donations is extremely important, as it switches the company's focus from making users mindlessly scroll through content to \
            generate more ad-revenue to providing a great user experience that want to contribute to with donations. Furthermore, not relying on ads means there is a much \
            greater incentive to deal with bots, as inflating the number of users and generated content becomes less relevant."
        </p>
        <p class="text-justify">
            "In such a structure, transparency is key, to show users that their donations are not misused. ShareSphere will always report how much donations it received \
            and how this money is used, for instance for operating costs and salaries. ShareSphere is also open source, enabling the community to know how the site \
            functions and which information is collected. Finally, ShareSphere aims to have transparent moderation, without shadow bans or unexplained content removal."
        </p>
        <p class="text-justify">
            "Another long term goal of ShareSphere is to give more control to the communities, by enabling them to select their moderators, define their rules and \
            leverage their help to deal with bots and bad actors. This is a long term idea and its implementation is not yet defined but the basic idea is to have a vote \
            based system that gives more weight to strong and regular contributors of each community to avoid communities being hijacked."
        </p>
        <p class="text-justify">
            "ShareSphere is still in its beginnings and additional features will come in the future, such as direct messages, better moderation tools, additional filters \
            and configuration and many more. I hope you enjoy ShareSphere and will help us grow into the best possible platform."
        </p>
    }
}

#[component]
pub fn Rules() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    view! {
        <div class="w-full overflow-y-auto">
            <div class="flex flex-col gap-4 items-center w-4/5 2xl:w-1/2 3xl:w-2/5 mx-auto py-4">
                <h1 class="text-3xl font-bold text-center">"Rules"</h1>
                <p class="text-justify">
                    "ShareSphere is a collaborative platform that relies on quality contributions from its users to thrive. \
                    Each community can decide upon its own set of rules but a set of base rules is enforced site-wide to ensure \
                    all communities remain safe, welcoming and compatible with ShareSphere's values."
                </p>
                <Suspense fallback=move || view! { <LoadingIcon/> }.into_any()>
                {
                    move || Suspend::new(async move {
                        match &state.base_rules.await {
                            Ok(rule_vec) => {
                                Either::Left(rule_vec.iter().enumerate().map(|(index, rule)| view! {
                                    <div class="flex flex-col gap-2">
                                        <h2 class="text-xl font-semibold">{format!("{}. {}", index + 1, rule.title)}</h2>
                                        <ContentBody
                                            body=rule.description.clone()
                                            is_markdown=rule.markdown_description.is_some()
                                            attr:class="text-justify"
                                        />
                                    </div>
                                }).collect_view())
                            },
                            Err(e) => Either::Right(view! { <ErrorDisplay error=e.clone()/> } ),
                        }
                    })
                }
                </Suspense>
            </div>
        </div>
        <HomeSidebar/>
    }
}

#[component]
fn ShareSphereInfo() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center gap-1">
            <p>"Effective Date: 01.06.2025"</p>
            <p>"Operator: ShareSphere"</p>
        </div>
    }
}

#[component]
fn AcceptanceOfTerms() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"1. Acceptance of Terms"</h2>
            <p class="text-justify">
                "By accessing or using ShareSphere (“we”, “us”, or “the Website”), you agree to be bound by these Terms and Conditions. \
                If you do not agree, please do not use the Website."
            </p>
        </div>
    }
}

#[component]
fn DescriptionOfService() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"2. Description of Service"</h2>
            <p class="text-justify">"ShareSphere provides an online platform for users to create, join, and participate in discussion forums"</p>
        </div>
    }
}

#[component]
fn UserResponsabilities() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"3. User Responsibilities"</h2>
            <p>"You agree not to:"</p>
            <ul class="list-disc list-inside text-justify">
                <li>"Post illegal, harmful, offensive, or misleading content."</li>
                <li>"Infringe ShareSphere's " <a href=RULES_ROUTE class="link text-primary">"Rules"</a>
                    ", ShareSphere's " <a href=CONTENT_POLICY_ROUTE class="link text-primary">"Content Policy"</a>
                    " and specific community rules."
                </li>
                <li>"Infringe on third-party rights, including copyright and data protection laws."</li>
                <li>"Use the site to distribute spam, malware, or phishing links."</li>
                <li>"Impersonate others or create multiple accounts for abuse."</li>
            </ul>
            <p>"You are solely responsible for all content you post."</p>
        </div>
    }
}

#[component]
fn Moderation() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"4. Moderation"</h2>
            <p class="text-justify">
                "We reserve the right to remove any content that violates these Terms or applicable laws and \
                to suspend or terminate user accounts without prior notice for misconduct."
            </p>
        </div>
    }
}

#[component]
fn IntellectualProperty() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"5. Intellectual Property"</h2>
            <p class="text-justify">"We retain ownership of all site content. Users grant a non-exclusive license to display their posts."</p>
        </div>
    }
}

#[component]
fn LimitationOfLiability() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"6. Limitation of Liability"</h2>
            <p class="text-justify">"We do not guarantee uninterrupted access or error-free operation of the Website. We are not liable for:"</p>
            <ul class="list-disc list-inside text-justify">
                <li>"User-generated content."</li>
                <li>"Loss of data, revenue, or reputation due to site use."</li>
                <li>"Any third-party actions or links accessed through the platform."</li>
            </ul>
        </div>
    }
}

#[component]
fn DataProtection() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"7. Data Protection"</h2>
            <p>
                "See our "
                <a href=PRIVACY_POLICY_ROUTE class="link text-primary">"Privacy Policy"</a>
                "."
            </p>
        </div>
    }
}

#[component]
fn Amendments() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"8. Amendments"</h2>
            <p class="text-justify">"We may update these Terms at any time. You will be notified of significant changes. Continued use of the Website means you accept the revised Terms."</p>
        </div>
    }
}

#[component]
fn GoverningLaw() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"9. Governing Law"</h2>
            <p class="text-justify">"These Terms are governed by Swiss law. Jurisdiction: Zurich."</p>
        </div>
    }
}

#[component]
fn AboutPrivacyPolicy() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"1. About ShareSphere's Privacy Policy"</h2>
            <p class="text-justify">"This Privacy Policy explains how we collect, use, and protect your personal data when you use ShareSphere."</p>
        </div>
    }
}

#[component]
fn DataCollection() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"2. Data ShareSphere Collects"</h2>
            <p class="text-justify">"ShareSphere collects the following information:"</p>
            <ul class="list-disc list-inside">
                <li>"Account data: username, email address, password (encrypted)."</li>
                <li>"IP address (for security)."</li>
                <li>"Any content you post."</li>
                <li>"Cookies for functionality."</li>
            </ul>
        </div>
    }
}

#[component]
fn DataCollectionPurpose() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"3. Purpose of Data Collection"</h2>
            <p class="text-justify">"ShareSphere uses your data to:"</p>
            <ul class="list-disc list-inside text-justify">
                <li>"Provide forum services."</li>
                <li>"Ensure security and prevent abuse."</li>
                <li>"Communicate with users."</li>
                <li>"Analyze usage and improve the site."</li>
            </ul>
        </div>
    }
}

#[component]
fn LegalBasis() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"4. Legal Basis"</h2>
            <p >"ShareSphere processes personal data based on:"</p>
            <ul class="list-disc list-inside text-justify">
                <li>"Your consent (e.g., when registering)."</li>
                <li>"Our legitimate interest in running a secure forum."</li>
                <li>"Communicate with users."</li>
                <li>"Compliance with legal obligations."</li>
            </ul>
        </div>
    }
}

#[component]
fn Cookies() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"5. Cookies"</h2>
            <p >"ShareSphere uses cookies for:"</p>
            <ul class="list-disc list-inside">
                <li>"Login sessions."</li>
            </ul>
        </div>
    }
}

#[component]
fn DataSharing() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"6. Data Sharing"</h2>
            <p class="text-justify">"ShareSphere does not sell your data. If required, ShareSphere might share your data with:"</p>
            <ul class="list-disc list-inside">
                <li>"Authorities."</li>
                <li>"Hosting providers."</li>
            </ul>
        </div>
    }
}

#[component]
fn DataStorage() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"7. Data Storage"</h2>
            <p class="text-justify">"ShareSphere stores data on server located in Switzerland following industry-standard encryption and security practices."</p>
        </div>
    }
}

#[component]
fn UserRights() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"8. Your rights"</h2>
            <p class="text-justify">"You have the right to:"</p>
            <ul class="list-disc list-inside text-justify">
                <li>"Access your personal data."</li>
                <li>"Request correction or deletion."</li>
                <li>"Withdraw consent at any time."</li>
                <li>"Lodge a complaint with the Swiss Federal Data Protection and Information Commissioner (FDPIC)."</li>
            </ul>
            <p class="text-justify">"To exercise these rights, email us at help@sharesphere.space"</p>
        </div>
    }
}

#[component]
fn PrivacyPolicyChanges() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"9. Changes to This Policy"</h2>
            <p class="text-justify">"We may update this Privacy Policy. We will notify you of significant changes."</p>
        </div>
    }
}