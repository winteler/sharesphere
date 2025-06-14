use leptos::either::Either;
use leptos::prelude::*;
use sharesphere_core::sidebar::HomeSidebar;
use sharesphere_utils::routes::PRIVACY_POLICY_ROUTE;

use sharesphere_core::rule::{get_rule_vec};
use sharesphere_utils::errors::{ErrorDisplay};
use sharesphere_utils::icons::{LoadingIcon, NsfwIcon, SpoilerIcon};
use sharesphere_utils::widget::ContentBody;

#[component]
pub fn TermsAndConditions() -> impl IntoView {
    view! {
        <div class="w-full overflow-y-auto">
            <div class="flex flex-col gap-4 items-center w-4/5 2xl:w-2/5 mx-auto py-4">
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
            <div class="flex flex-col gap-4 items-center w-4/5 2xl:w-2/5 mx-auto py-4">
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
            <div class="flex flex-col gap-4 w-4/5 2xl:w-2/5 mx-auto py-4">
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
                    <div>
                        "Content that is not suitable for minors, such as sexually explicit, graphic, violent or offensive contents must be labelled with the 'NSFW' tag: "
                        <NsfwIcon class="inline-flex"/>
                    </div>
                    <h3 class="text-lg font-semibold">"Spoiler Content"</h3>
                    <div>
                        "Content that could spoil information to other users, such as the content of a book or movie \
                        must keep any information out of its title and has to be labelled with the 'Spoiler' tag: "
                        <div class="h-fit w-fit px-1 py-0.5 bg-black rounded-full inline-flex relative top-1"><SpoilerIcon/></div>
                    </div>
                    <p>"The labelling rules for different content types are as follows:"</p>
                    <ul class="list-disc list-inside">
                        <li>"Any plot relevant information of books, movie, games, TV shows must always be labelled as spoiler."</li>
                        <li>"Results of competitions (sports, e-sport, games, etc.) and other live events must be labelled as spoiler in the week following the result."</li>
                    </ul>
                    <p>
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
pub fn Rules() -> impl IntoView {
    let rule_vec_resource = OnceResource::new(get_rule_vec(None));
    view! {
        <div class="w-full overflow-y-auto">
            <div class="flex flex-col gap-4 items-center w-4/5 2xl:w-2/5 mx-auto py-4">
                <h1 class="text-3xl font-bold text-center">"Rules"</h1>
                <p class="text-justify">
                    "ShareSphere is a collaborative platform that relies on quality contributions from its users to thrive. \
                    Each community can decide upon its own set of rules but a set of base rules is enforced site-wide to ensure \
                    all communities remain safe, welcoming and compatible with ShareSphere's values."
                </p>
                <Suspense fallback=move || view! { <LoadingIcon/> }.into_any()>
                {
                    move || Suspend::new(async move {
                        match &rule_vec_resource.await {
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
            <p >
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
            <p >"ShareSphere provides an online platform for users to create, join, and participate in discussion forums"</p>
        </div>
    }
}

#[component]
fn UserResponsabilities() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"3. User Responsibilities"</h2>
            <p>"You agree not to:"</p>
            <ul class="list-disc list-inside">
                <li>"Post illegal, harmful, offensive, or misleading content."</li>
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
            <p >
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
            <p >"We retain ownership of all site content. Users grant a non-exclusive license to display their posts."</p>
        </div>
    }
}

#[component]
fn LimitationOfLiability() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"8. Amendments"</h2>
            <p >"We do not guarantee uninterrupted access or error-free operation of the Website."</p>
            <p >"We are not liable for:"</p>
            <ul class="list-disc list-inside">
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
            <h2 class="text-2xl font-semibold">"9. Governing Law"</h2>
            <p >
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
            <p >"We may update these Terms at any time. You will be notified of significant changes. Continued use of the Website means you accept the revised Terms."</p>
        </div>
    }
}

#[component]
fn GoverningLaw() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"9. Governing Law"</h2>
            <p >"These Terms are governed by Swiss law. Jurisdiction: Zurich."</p>
        </div>
    }
}

#[component]
fn AboutPrivacyPolicy() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"1. About ShareSphere's Privacy Policy"</h2>
            <p >"This Privacy Policy explains how we collect, use, and protect your personal data when you use ShareSphere."</p>
        </div>
    }
}

#[component]
fn DataCollection() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"2. Data ShareSphere Collects"</h2>
            <p >"ShareSphere collects the following information:"</p>
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
            <p >"ShareSphere uses your data to:"</p>
            <ul class="list-disc list-inside">
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
            <ul class="list-disc list-inside">
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
            <p >"ShareSphere does not sell your data. If required, ShareSphere might share your data with:"</p>
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
            <p >"ShareSphere stores data on server located in Switzerland following industry-standard encryption and security practices."</p>
        </div>
    }
}

#[component]
fn UserRights() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"8. Your rights"</h2>
            <p >"You have the right to::"</p>
            <ul class="list-disc list-inside">
                <li>"Access your personal data."</li>
                <li>"Request correction or deletion."</li>
                <li>"Withdraw consent at any time."</li>
                <li>"Lodge a complaint with the Swiss Federal Data Protection and Information Commissioner (FDPIC)."</li>
            </ul>
            <p >"To exercise these rights, email us at help@sharesphere.space"</p>
        </div>
    }
}

#[component]
fn PrivacyPolicyChanges() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"9. Changes to This Policy"</h2>
            <p >"We may update this Privacy Policy. We will notify you of significant changes."</p>
        </div>
    }
}