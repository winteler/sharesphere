use leptos::prelude::*;
use sharesphere_core::sidebar::HomeSidebar;
use sharesphere_utils::routes::PRIVACY_POLICY_ROUTE;

#[component]
pub fn TermsAndConditions() -> impl IntoView {
    view! {
        <div class="w-full overflow-y-auto">
            <div class="flex flex-col gap-3 items-center w-full 2xl:w-2/5 mx-auto">
                <h1 class="text-3xl font-bold text-center">"Terms and Conditions"</h1>
                <ShareSpherInfo/>
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
            <div class="flex flex-col gap-3 items-center w-full 2xl:w-2/5 mx-auto">
                <h1 class="text-3xl font-bold text-center">"Privacy Policy"</h1>
                <ShareSpherInfo/>
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
pub fn Rules() -> impl IntoView {
    view! {
        <div class="w-full overflow-y-auto">
            <div class="flex flex-col gap-3 items-center w-full 2xl:w-2/5 mx-auto">
                <h1 class="text-3xl font-bold text-center">"Rules"</h1>
            </div>
        </div>
        <HomeSidebar/>
    }
}

#[component]
fn ShareSpherInfo() -> impl IntoView {
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
            <p class="">
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
            <p class="">"ShareSphere provides an online platform for users to create, join, and participate in discussion forums"</p>
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
            <p class="">
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
            <p class="">"We retain ownership of all site content. Users grant a non-exclusive license to display their posts."</p>
        </div>
    }
}

#[component]
fn LimitationOfLiability() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"8. Amendments"</h2>
            <p class="">"We do not guarantee uninterrupted access or error-free operation of the Website."</p>
            <p class="">"We are not liable for:"</p>
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
            <p class="">
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
            <p class="">"We may update these Terms at any time. You will be notified of significant changes. Continued use of the Website means you accept the revised Terms."</p>
        </div>
    }
}

#[component]
fn GoverningLaw() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"9. Governing Law"</h2>
            <p class="">"These Terms are governed by Swiss law. Jurisdiction: Zurich."</p>
        </div>
    }
}

#[component]
fn AboutPrivacyPolicy() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"1. About ShareSphere's Privacy Policy"</h2>
            <p class="">"This Privacy Policy explains how we collect, use, and protect your personal data when you use ShareSphere."</p>
        </div>
    }
}

#[component]
fn DataCollection() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"2. Data ShareSphere Collects"</h2>
            <p class="">"ShareSphere collects the following information:"</p>
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
            <p class="">"ShareSphere uses your data to:"</p>
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
            <p class="">"ShareSphere processes personal data based on:"</p>
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
            <p class="">"ShareSphere uses cookies for:"</p>
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
            <p class="">"ShareSphere does not sell your data. If required, ShareSphere might share your data with:"</p>
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
            <p class="">"ShareSphere stores data on server located in Switzerland following industry-standard encryption and security practices."</p>
        </div>
    }
}

#[component]
fn UserRights() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"8. Your rights"</h2>
            <p class="">"You have the right to::"</p>
            <ul class="list-disc list-inside">
                <li>"Access your personal data."</li>
                <li>"Request correction or deletion."</li>
                <li>"Withdraw consent at any time."</li>
                <li>"Lodge a complaint with the Swiss Federal Data Protection and Information Commissioner (FDPIC)."</li>
            </ul>
            <p class="">"To exercise these rights, email us at help@sharesphere.space"</p>
        </div>
    }
}

#[component]
fn PrivacyPolicyChanges() -> impl IntoView {
    view! {
        <div class="w-full flex flex-col gap-1">
            <h2 class="text-2xl font-semibold">"9. Changes to This Policy"</h2>
            <p class="">"We may update this Privacy Policy. We will notify you of significant changes."</p>
        </div>
    }
}