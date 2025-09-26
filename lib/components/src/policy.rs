use leptos::either::Either;
use leptos::prelude::*;
use leptos_fluent::move_tr;
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
            <div class="flex flex-col gap-4 w-4/5 lg:w-1/2 3xl:w-2/5 mx-auto py-4">
                <h1 class="text-3xl font-bold text-center">{move_tr!("about-sharesphere")}</h1>
                <p class="text-justify">
                    {move_tr!("about-sharesphere-content")}
                </p>
                <h2 class="text-xl font-semibold">{move_tr!("rules-and-moderation")}</h2>
                <p class="text-justify">
                    {move_tr!("about-sharesphere-rules-1")}
                    <a href=RULES_ROUTE>{move_tr!("about-sharesphere-rules-link")}</a>
                    {move_tr!("about-sharesphere-rules-2")}
                </p>
                <PlannedImprovements/>
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
            <div class="flex flex-col gap-4 items-center w-4/5 lg:w-1/2 3xl:w-2/5 mx-auto py-4">
                <h1 class="text-3xl font-bold text-center">{move_tr!("terms-and-condition")}</h1>
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
            <div class="flex flex-col gap-4 items-center w-4/5 lg:w-1/2 3xl:w-2/5 mx-auto py-4">
                <h1 class="text-3xl font-bold text-center">{move_tr!("privacy-policy")}</h1>
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
            <div class="flex flex-col gap-4 w-4/5 lg:w-1/2 3xl:w-2/5 mx-auto py-4">
                <h1 class="text-3xl font-bold text-center">{move_tr!("content-policy")}</h1>
                <p class="text-justify">
                    {move_tr!("content-policy-intro")}
                </p>
                <div class="flex flex-col gap-2">
                    <h2 class="text-xl font-semibold">{move_tr!("banned-content-title")}</h2>
                    <p>{move_tr!("banned-content-intro")}</p>
                    <ul class="list-disc list-inside">
                        <li>{move_tr!("banned-content-1")}</li>
                        <li>{move_tr!("banned-content-2")}</li>
                        <li>{move_tr!("banned-content-3")}</li>
                        <li>{move_tr!("banned-content-4")}</li>
                        <li>{move_tr!("banned-content-5")}</li>
                        <li>{move_tr!("banned-content-6")}</li>
                        <li>{move_tr!("banned-content-7")}</li>
                        <li>{move_tr!("banned-content-8")}</li>
                        <li>{move_tr!("banned-content-9")}</li>
                        <li>{move_tr!("banned-content-10")}</li>
                    </ul>
                </div>
                <div class="flex flex-col gap-2">
                    <h2 class="text-xl font-semibold">{move_tr!("sensitive-content-title")}</h2>
                    <h3 class="text-lg font-semibold">{move_tr!("mature-content-title")}</h3>
                    <div class="text-justify">
                        {move_tr!("mature-content-description")}
                        <NsfwIcon class="inline-flex"/>
                    </div>
                    <h3 class="text-lg font-semibold">{move_tr!("spoiler-content-title")}</h3>
                    <div class="text-justify">
                        {move_tr!("spoiler-content-description")}
                        <div class="h-fit w-fit px-1 py-0.5 bg-black rounded-full inline-flex relative top-1"><SpoilerIcon/></div>
                    </div>
                    <p>{move_tr!("spoiler-content-label-1")}</p>
                    <ul class="list-disc list-inside text-justify">
                        <li>{move_tr!("spoiler-content-label-2")}</li>
                        <li>{move_tr!("spoiler-content-label-3")}</li>
                    </ul>
                    <p class="text-justify">{move_tr!("spoiler-content-label-4")}</p>
                </div>
            </div>
        </div>
        <HomeSidebar/>
    }
}

#[component]
pub fn PlannedImprovements() -> impl IntoView {
    view! {
        <h2 class="text-xl font-semibold">{move_tr!("planned-improvements-title")}</h2>
        <ul class="list-disc list-inside text-justify">
            <li>{move_tr!("planned-improvements-1")}</li>
            <li>{move_tr!("planned-improvements-2")}</li>
            <li>{move_tr!("planned-improvements-3")}</li>
            <li>{move_tr!("planned-improvements-4")}</li>
            <li>{move_tr!("planned-improvements-5")}</li>
        </ul>
    }
}
#[component]
pub fn OriginsAndGoals() -> impl IntoView {
    view! {
        <h2 class="text-xl font-semibold">{move_tr!("origin-goals-title")}</h2>
        <p class="text-justify">{move_tr!("origin-goals-1")}</p>
        <p class="text-justify">{move_tr!("origin-goals-2")}</p>
        <p class="text-justify">{move_tr!("origin-goals-3")}</p>
        <p class="text-justify">{move_tr!("origin-goals-4")}</p>
        <p class="text-justify">{move_tr!("origin-goals-5")}</p>
    }
}

#[component]
pub fn Rules() -> impl IntoView {
    let state = expect_context::<GlobalState>();
    view! {
        <div class="w-full overflow-y-auto">
            <div class="flex flex-col gap-4 items-center w-4/5 lg:w-1/2 3xl:w-2/5 mx-auto py-4">
                <h1 class="text-3xl font-bold text-center">{move_tr!("rules")}</h1>
                <p class="text-justify">{move_tr!("rules-intro")}</p>
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
            <p>{move_tr!("info-validity")}</p>
            <p>{move_tr!("info-operator")}</p>
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
            <p class="text-justify">"All content on ShareSphere, except user submissions, is the property of ShareSphere and may not be used without permission."</p>
            <p class="text-justify">"By posting content, you grant us a non-exclusive, royalty-free, worldwide license to use, display, and distribute your content on the platform."</p>
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