//! The vocabulary.
//!
//! Every term here exists because schema.org has no word for something a
//! Bundu standard needs to say. That test is deliberately strict: if
//! schema.org can express it, we use schema.org and add nothing. A
//! vocabulary earns trust by being small.
//!
//! # This namespace is not about identity
//!
//! `vocab.bundu.org` serves terms for *every* Bundu standard, across every
//! domain and sector. Identity and trust happen to be first because that is
//! what was built first. A standard about health records, agricultural
//! supply, or education delivery would publish its terms here too.
//!
//! Terms therefore carry a [`Domain`] for grouping, and URIs stay flat —
//! `vocab.bundu.org/Family`, never `vocab.bundu.org/identity/Family`. A term
//! URI is a permanent contract the moment someone embeds it in their data,
//! and path segments encode a taxonomy that will eventually be wrong.
//! schema.org has over 800 flat terms for the same reason.
//!
//! Each term carries a `rationale` explaining what was tried first and why it
//! did not fit, so anyone auditing the model later can judge whether the term
//! is still needed rather than guessing at intent.

/// Whether a term is a class (a kind of thing) or a property (something a
/// thing has).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Class,
    Property,
    /// A named instance of a class: one rule of a standard, or the standard's
    /// own set of rules. Its RDF types come from [`Individual::types`].
    Individual,
}

impl Kind {
    /// The RDF type URI for this kind of term.
    pub const fn rdf_type(self) -> &'static str {
        match self {
            Kind::Class => "rdfs:Class",
            Kind::Property => "rdf:Property",
            Kind::Individual => "rdfs:Resource",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Kind::Class => "Class",
            Kind::Property => "Property",
            Kind::Individual => "Individual",
        }
    }
}

/// How settled a term is, published as W3C `vs:term_status`.
///
/// A published URI never disappears, so status is the only honest way to say
/// "this may still change". A term starts `Pending` when the standard that
/// needs it is still being written, and becomes `Stable` once that standard
/// is published or the term is in live data. A stable term's meaning never
/// changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// In live data or a published standard. Its meaning is fixed.
    Stable,
    /// Its standard is still being written. The URI is permanent, but the
    /// definition may still be refined before the term becomes stable.
    /// No term is pending today; the next standard's new terms start here.
    #[allow(dead_code)]
    Pending,
}

impl Status {
    /// The label people read, on the term page and in the index.
    pub const fn label(self) -> &'static str {
        match self {
            Status::Stable => "stable",
            Status::Pending => "pending",
        }
    }

    /// The W3C vocabulary-status value (`vs:term_status`). The W3C set is
    /// stable, testing, unstable and archaic; `testing` is the one that means
    /// "published so it can be tried, and may still be refined".
    pub const fn vs_term_status(self) -> &'static str {
        match self {
            Status::Stable => "stable",
            Status::Pending => "testing",
        }
    }
}

/// Subject area, for grouping in the index only.
///
/// Adding a variant is how the vocabulary grows into a new sector. It has no
/// effect on term URIs, which stay flat and permanent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    /// Who someone is, and which collectives they belong to.
    IdentityAndKinship,
    /// How standing is established and evidenced.
    TrustAndRecognition,
    /// Trade that happens outside formal registration.
    InformalEconomy,
    /// Publishers, mastheads and the desks inside them.
    NewsAndMedia,
    /// Data given back to the commons, and the measures that keep it
    /// anonymous.
    OpenData,
    /// Rules that constrain what an AI system may say or do.
    AiGuardrails,
}

impl Domain {
    pub const fn label(self) -> &'static str {
        match self {
            Domain::IdentityAndKinship => "Identity and kinship",
            Domain::TrustAndRecognition => "Trust and recognition",
            Domain::InformalEconomy => "Informal economy",
            Domain::NewsAndMedia => "News and media",
            Domain::OpenData => "Open data",
            Domain::AiGuardrails => "AI guardrails",
        }
    }

    /// Declaration order controls index order.
    pub const ALL: &'static [Domain] = &[
        Domain::IdentityAndKinship,
        Domain::TrustAndRecognition,
        Domain::InformalEconomy,
        Domain::NewsAndMedia,
        Domain::OpenData,
        Domain::AiGuardrails,
    ];
}

/// A literal value an individual states about itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Literal {
    Text(&'static str),
    Boolean(bool),
}

/// The facts that make an individual identifiable: what it is, its code in
/// the set it belongs to, and the values it holds.
#[derive(Debug, Clone, Copy)]
pub struct Individual {
    /// Its RDF types. At least one is a schema.org class, so a consumer that
    /// does not know this vocabulary still reads something.
    pub types: &'static [&'static str],
    /// `schema:termCode`, where the individual is a DefinedTerm.
    pub term_code: Option<&'static str>,
    /// `schema:inDefinedTermSet`, where it belongs to one.
    pub in_set: Option<&'static str>,
    /// Further (property, value) facts, such as a rule's category.
    pub facts: &'static [(&'static str, Literal)],
}

/// A single vocabulary term.
pub struct Term {
    /// The term name, and the last path segment of its URI.
    pub id: &'static str,
    pub kind: Kind,
    pub domain: Domain,
    pub status: Status,
    pub label: &'static str,
    /// What the term means.
    pub comment: &'static str,
    /// Parent class, for classes. Either a schema.org class or another class
    /// in this vocabulary whose own parent chain reaches schema.org.
    pub sub_class_of: Option<&'static str>,
    /// What a property attaches to.
    pub domain_includes: Option<&'static str>,
    /// What a property's value is.
    pub range: Option<&'static str>,
    /// Permitted values, where the property is an enumeration.
    pub values: Option<&'static [(&'static str, &'static str)]>,
    /// What an individual is, for individuals only.
    pub individual: Option<Individual>,
    /// Why schema.org was insufficient. The justification for the term
    /// existing at all.
    pub rationale: &'static str,
}

/// Permitted values for [`recognitionBasis`](TERMS).
const RECOGNITION_BASIS_VALUES: &[(&str, &str)] = &[
    (
        "customary",
        "Standing derives from custom and is attested by a recognised \
         authority. No registry entry exists or is expected.",
    ),
    (
        "statutory",
        "A statute names this kind of entity and confers standing directly, \
         without requiring registration.",
    ),
    (
        "registry",
        "Standing derives from an entry in a public register, which is the \
         evidence.",
    ),
    (
        "member_consent",
        "Standing derives only from the agreement of its members. No external \
         record exists, and none will.",
    ),
    (
        "contested",
        "More than one authority claims jurisdiction, or standing is disputed. \
         Recorded rather than resolved.",
    ),
    ("none", "No recognised basis. Distinct from unknown."),
];

/// Permitted values for [`attestation`](TERMS). The live set in the Nyuchi
/// API (`gateway/lib/assurance.py`); adding a value is additive.
const ATTESTATION_VALUES: &[(&str, &str)] = &[
    (
        "community",
        "Enough people in the ecosystem have vouched for the subject, each \
         staking their own standing on it. Granted automatically at the \
         threshold, and costs the subject no paperwork.",
    ),
    (
        "business",
        "A business registration reference was checked against its register.",
    ),
    (
        "domain",
        "Control of an internet domain was shown, by a DNS TXT record or a \
         mailbox on the domain.",
    ),
];

/// Permitted values for [`guardrailSeverity`](TERMS). The values the live
/// guardrail store already validates.
const GUARDRAIL_SEVERITY_VALUES: &[(&str, &str)] = &[
    (
        "critical",
        "Breaching the rule could cause serious harm to a person. The system \
         refuses, whatever the surface or the request.",
    ),
    (
        "high",
        "Breaching the rule causes real harm or a serious breach of trust.",
    ),
    (
        "medium",
        "Breaching the rule is a material lapse in quality, accuracy or tone.",
    ),
    (
        "low",
        "Guidance on style or tone. A breach is a lapse, not a harm.",
    ),
];

/// Permitted values for [`guardrailCategory`](TERMS): the categories of the
/// Bundu AI Guardrails Standard 1.0.0. Owner-approved 2026-10-04. The
/// descriptions here are the vocabulary's own; the standard's wording is
/// published with the standard under CC BY 4.0.
const GUARDRAIL_CATEGORY_VALUES: &[(&str, &str)] = &[
    ("child_safety", "The safety of children and minors."),
    ("self_harm", "Self-harm, suicide and eating disorders."),
    ("violent_extremism", "Terrorism and violent extremism."),
    (
        "hate_harassment",
        "Hatred, harassment, bullying and threats against people.",
    ),
    ("sexual_content", "Sexually explicit content, and consent."),
    (
        "weapons_illegal",
        "Weapons, serious harm and serious crime.",
    ),
    (
        "privacy",
        "Personal data: extracting, revealing, inferring or collecting it.",
    ),
    (
        "prompt_injection",
        "Attempts to override the system's instructions, or to extract its \
         prompt, keys or secrets.",
    ),
    (
        "impersonation_misinformation",
        "Honesty about being an AI, impersonation, and falsehoods.",
    ),
    (
        "professional_advice",
        "Medical, legal and financial advice.",
    ),
    (
        "cultural_respect",
        "Respect for cultures, traditions and languages, African ones in \
         particular, and Ubuntu.",
    ),
    (
        "refusal_style",
        "How the system says no when a rule stops it.",
    ),
];

/// Why each published rule gets a URI. Shared by every rule: the reason is
/// the same for all of them, and the rule's own wording lives in the
/// standard, not here.
const BAGS_RULE_RATIONALE: &str = "A published rule needs a URI that never \
    changes, so that a policy, an audit, an AI system card or a guardrail \
    store can cite exactly this rule rather than a paraphrase of it. The \
    code is final for BAGS 1.0.0 (owner-approved 2026-10-04), and a later \
    edition adds codes rather than reusing one. The rule's full wording is \
    published with the standard under CC BY 4.0 and is not copied into this \
    CC0 vocabulary.";

/// Types every BAGS 1.0.0 rule carries.
const BAGS_RULE_TYPES: &[&str] = &["schema:DefinedTerm", "bundu:PlatformGuardrail"];

/// The full vocabulary.
pub const TERMS: &[Term] = &[
    Term {
        id: "Family",
        kind: Kind::Class,
        domain: Domain::IdentityAndKinship,
        status: Status::Stable,
        label: "Family",
        comment: "A kinship group treated as a first-class entity: it holds a \
                  handle, a verification standing and members, exactly as an \
                  organization does. In the Bundu model a person cannot exist \
                  without belonging to one.",
        sub_class_of: Some("schema:Organization"),
        domain_includes: None,
        range: None,
        values: None,
        individual: None,
        rationale: "schema.org has no Family class. schema:Organization is \
                    structurally correct — a family has members, a name and an \
                    identity — but it carries commercial connotations that \
                    misrepresent what a family is, and consuming systems infer \
                    business semantics from it. Families are serialised as \
                    schema:Organization with additionalType pointing here, so \
                    the JSON-LD stays valid for consumers that do not know this \
                    vocabulary while remaining truthful for those that do.",
    },
    Term {
        id: "Community",
        kind: Kind::Class,
        domain: Domain::IdentityAndKinship,
        status: Status::Stable,
        label: "Community",
        comment: "A group organised around a shared place, practice or purpose \
                  that is neither a family nor a registered organization. \
                  Savings groups, burial societies, cooperatives and \
                  neighbourhood associations.",
        sub_class_of: Some("schema:Organization"),
        domain_includes: None,
        range: None,
        values: None,
        individual: None,
        rationale: "Communities in this ecosystem are frequently unregistered \
                    and derive standing from collective attestation rather than \
                    from a registry. schema:Organization implies a formality \
                    most of them do not have and should not be required to \
                    perform in order to be represented. Deliberately broad: a \
                    South African stokvel and a Kenyan chama are the same \
                    structure under two names, and typing them separately would \
                    make them incomparable for no reason.",
    },
    Term {
        id: "PlaceOwner",
        kind: Kind::Class,
        domain: Domain::IdentityAndKinship,
        status: Status::Stable,
        label: "Place Owner",
        comment: "An entity whose relationship to the ecosystem is custody of a \
                  physical place — the party answerable for a venue, market \
                  stall, farm or landmark.",
        sub_class_of: Some("schema:Organization"),
        domain_includes: None,
        range: None,
        values: None,
        individual: None,
        rationale: "schema.org models places and organizations, but not the \
                    custodial relationship between them as a kind of entity. \
                    Custody matters here because it is what a verification \
                    claim attaches to when a place is claimed.",
    },
    Term {
        id: "recognitionBasis",
        kind: Kind::Property,
        domain: Domain::TrustAndRecognition,
        status: Status::Stable,
        label: "recognition basis",
        comment: "What makes an entity legitimate, and therefore what evidence \
                  is meaningful when verifying it.",
        sub_class_of: None,
        domain_includes: Some("schema:Organization"),
        range: Some("schema:Text"),
        values: Some(RECOGNITION_BASIS_VALUES),
        individual: None,
        rationale: "Structurally identical collectives have wildly different \
                    legal standing. A South African stokvel is defined in \
                    statute but is not a juristic person; an Indonesian desa \
                    adat is a full public legal entity; a Mexican ejido has a \
                    registry entry; a Zimbabwean village under a headman has \
                    constitutional recognition undercut by two conflicting \
                    Acts. Without this property, an entirely real unregistered \
                    entity is indistinguishable from a registered one with \
                    missing paperwork — so verification demands a registration \
                    number that does not and never will exist. That is a \
                    category error, not a failed check. Modelling the basis of \
                    legitimacy as one property avoids enumerating every local \
                    legal form: ISO 20275 took that path and reached 3,400 \
                    forms across 185 jurisdictions while still being described \
                    by its own maintainers as a work in progress.",
    },
    Term {
        id: "verificationTier",
        kind: Kind::Property,
        domain: Domain::TrustAndRecognition,
        status: Status::Stable,
        label: "verification tier",
        comment: "How far an entity has progressed through verification. \
                  0 = unclaimed, 1 = claimed, 2 = verified, 3 = fully verified.",
        sub_class_of: None,
        domain_includes: Some("schema:Thing"),
        range: Some("schema:Integer"),
        values: None,
        individual: None,
        rationale: "An ordered ladder, not a boolean. schema.org offers no \
                    graduated trust signal, and a plain verified/unverified flag \
                    cannot express the difference between an entity someone has \
                    claimed and one whose registration documents have been \
                    reviewed. Read alongside recognitionBasis, which says what \
                    climbing the ladder should require.",
    },
    Term {
        id: "ubuntuScore",
        kind: Kind::Property,
        domain: Domain::TrustAndRecognition,
        status: Status::Stable,
        label: "ubuntu score",
        comment: "A standing derived from an entity's conduct toward others in \
                  the ecosystem, rather than from customer satisfaction.",
        sub_class_of: None,
        domain_includes: Some("schema:Organization"),
        range: Some("schema:Number"),
        values: None,
        individual: None,
        rationale: "schema:aggregateRating measures how well something served \
                    the person rating it. This measures something different — \
                    reciprocity and contribution — and conflating the two would \
                    let a highly rated but extractive entity appear trustworthy.",
    },
    Term {
        id: "communityVouches",
        kind: Kind::Property,
        domain: Domain::TrustAndRecognition,
        status: Status::Stable,
        label: "community vouches",
        comment: "The number of entities that have vouched for this one, where \
                  vouching carries reputational consequence for the voucher.",
        sub_class_of: None,
        domain_includes: Some("schema:Organization"),
        range: Some("schema:Integer"),
        values: None,
        individual: None,
        rationale: "Distinct from schema:reviewCount because a vouch is not a \
                    review: the voucher stakes their own standing on the claim. \
                    In economies without reliable registries this is often the \
                    only trust signal available, and it must not be read as \
                    popularity.",
    },
    Term {
        id: "isInformal",
        kind: Kind::Property,
        domain: Domain::InformalEconomy,
        status: Status::Stable,
        label: "is informal",
        comment: "Whether an entity operates in the informal economy — trading \
                  without formal registration.",
        sub_class_of: None,
        domain_includes: Some("schema:Organization"),
        range: Some("schema:Boolean"),
        values: None,
        individual: None,
        rationale: "Most economic activity in the ecosystem's markets is \
                    informal. Recording this plainly, as a neutral fact rather \
                    than an absence of data, is what lets informal traders be \
                    represented at all instead of appearing as incomplete \
                    records of formal ones.",
    },
    // ── Trust and recognition: persons ─────────────────────────────────
    Term {
        id: "assuranceLevel",
        kind: Kind::Property,
        domain: Domain::TrustAndRecognition,
        status: Status::Stable,
        label: "assurance level",
        comment: "How strongly a person's identity has been proven. \
                  0 = none, 1 = otp (an email address or phone number was \
                  verified), 2 = documented (a non-government document was \
                  checked), 3 = government (a government-issued ID was \
                  verified), 4 = licensed (a professional licence was \
                  verified). Ordered, so a higher level includes the lower \
                  ones. Levels 2 to 4 map to NIST SP 800-63-4 identity \
                  assurance: 2 to IAL1, 3 and 4 to IAL2; 0 and 1 sit below \
                  IAL1. The mapping is a reference, not a claim of certified \
                  conformance.",
        sub_class_of: None,
        domain_includes: Some("schema:Person"),
        range: Some("schema:Integer"),
        values: None,
        individual: None,
        rationale: "schema.org has no identity-assurance property. NIST IAL \
                    alone is too coarse for the ecosystem: it has nothing \
                    below IAL1, and a verified phone number is a real, \
                    useful rung for a trader with no documents. Kept \
                    separate from verificationTier, which is the ladder an \
                    entity climbs, and from attestation, which is unordered: \
                    a person vouched for by their whole street is not ranked \
                    below one who clicked an emailed link.",
    },
    Term {
        id: "attestation",
        kind: Kind::Property,
        domain: Domain::TrustAndRecognition,
        status: Status::Stable,
        label: "attestation",
        comment: "A kind of standing that has been attested for a person or \
                  entity, independent of its assurance level. Repeatable: a \
                  subject can hold several. A revoked attestation is not \
                  published.",
        sub_class_of: None,
        domain_includes: Some("schema:Thing"),
        range: Some("schema:Text"),
        values: Some(ATTESTATION_VALUES),
        individual: None,
        rationale: "Assurance and attestation are deliberately not one \
                    number. An earlier single 0-4 ladder put community \
                    vouching below a verified email address, ranking a \
                    trader vouched for by their whole street below anyone \
                    who clicked a link. Attestations are parallel routes to \
                    trust: a tuck shop is otp plus community, a registered \
                    firm is government plus business, and neither outranks \
                    the other. schema.org has no property for either. \
                    schema:hasCredential names a credential, not the kind \
                    of standing it proves.",
    },
    // ── Identity and kinship: family first ─────────────────────────────
    Term {
        id: "defaultFamily",
        kind: Kind::Property,
        domain: Domain::IdentityAndKinship,
        status: Status::Stable,
        label: "default family",
        comment: "The Family a person belongs to first. Every person has \
                  one from their first sign-in, a family-of-one if nobody \
                  else has joined it yet, before any other membership.",
        sub_class_of: None,
        domain_includes: Some("schema:Person"),
        range: Some("bundu:Family"),
        values: None,
        individual: None,
        rationale: "In the Bundu model a person cannot exist without \
                    belonging to a family (the family-first invariant). \
                    schema:memberOf says that a person belongs to a family \
                    but not which of their memberships is the anchor, and a \
                    person can be a member of several families. Without this \
                    property a consumer cannot tell the family a person \
                    stands in from one they married into or were added to.",
    },
    // ── News and media ─────────────────────────────────────────────────
    Term {
        id: "PublishingHouse",
        kind: Kind::Class,
        domain: Domain::NewsAndMedia,
        status: Status::Stable,
        label: "Publishing House",
        comment: "A group that publishes several mastheads, such as \
                  Zimpapers or Alpha Media Holdings. Each masthead is a \
                  schema:NewsMediaOrganization whose parentOrganization is \
                  the house. A masthead is never its own house.",
        sub_class_of: Some("schema:NewsMediaOrganization"),
        domain_includes: None,
        range: None,
        values: None,
        individual: None,
        rationale: "schema.org types the masthead (NewsMediaOrganization) \
                    but not the group above it. Without a term, a house and \
                    its mastheads serialise identically, and a news \
                    pipeline that cannot tell them apart makes every \
                    masthead its own publisher: the Mukoko pipeline minted \
                    622 such self-parented records before the distinction \
                    was enforced. Serialised as NewsMediaOrganization with \
                    additionalType pointing here.",
    },
    Term {
        id: "NewsDesk",
        kind: Kind::Class,
        domain: Domain::NewsAndMedia,
        status: Status::Stable,
        label: "News Desk",
        comment: "A desk inside a masthead or publishing house, such as the \
                  business desk or the sports desk. Also called a media \
                  desk. Its parent is the masthead or house it sits in, and \
                  it is published as that parent's department.",
        sub_class_of: Some("schema:NewsMediaOrganization"),
        domain_includes: None,
        range: None,
        values: None,
        individual: None,
        rationale: "schema:department links an organisation to a part of \
                    itself, but schema.org has no type for that part in a \
                    newsroom. A desk is where bylines, beats and corrections \
                    are owned, so it needs to be an entity that journalists \
                    work for, not a free-text section name like \
                    schema:articleSection. Serialised as \
                    NewsMediaOrganization with additionalType pointing here.",
    },
    // ── Open data ──────────────────────────────────────────────────────
    Term {
        id: "kAnonymity",
        kind: Kind::Property,
        domain: Domain::OpenData,
        status: Status::Stable,
        label: "k-anonymity",
        comment: "The k a published dataset is guaranteed to meet: every \
                  row, and every group of rows sharing the same \
                  quasi-identifiers, describes at least k distinct people.",
        sub_class_of: None,
        domain_includes: Some("schema:Dataset"),
        range: Some("schema:Integer"),
        values: None,
        individual: None,
        rationale: "Open data given back by people who deleted their \
                    accounts is only safe to use if a re-user can see how \
                    anonymous it is. Neither schema.org nor DCAT has a \
                    property for an anonymity guarantee. W3C DPV names \
                    k-anonymity as a technique a controller uses, but not \
                    the k a published dataset meets, and the k is what a \
                    researcher needs in order to judge the data.",
    },
    Term {
        id: "maxReidentificationRisk",
        kind: Kind::Property,
        domain: Domain::OpenData,
        status: Status::Stable,
        label: "maximum re-identification risk",
        comment: "The highest probability, from 0 to 1, that a record in a \
                  published dataset can be linked to a known person by an \
                  attacker who knows that person contributed and knows \
                  their quasi-identifiers (the prosecutor model). For a \
                  k-anonymous release it is at most 1/k.",
        sub_class_of: None,
        domain_includes: Some("schema:Dataset"),
        range: Some("schema:Number"),
        values: None,
        individual: None,
        rationale: "A k value says how a release was built; the measured \
                    risk says what the check found when it was built. \
                    Publishing both lets anyone audit a release without \
                    trusting the publisher's method. schema.org, DCAT and \
                    DPV have no property for a measured re-identification \
                    risk.",
    },
    // ── AI guardrails (Bundu AI Guardrails Standard, stable) ──────────
    Term {
        id: "AIGuardrail",
        kind: Kind::Class,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "AI Guardrail",
        comment: "A rule that constrains what an AI system may say or do, \
                  with the guidance the system is given to follow it. A \
                  guardrail is either a platform guardrail or an entity \
                  guardrail. A published rule carries a code (its \
                  schema:termCode, such as BAGS-01) in the set of rules it \
                  belongs to. Defined by the Bundu AI Guardrails Standard \
                  (BAGS).",
        sub_class_of: Some("schema:DefinedTerm"),
        domain_includes: None,
        range: None,
        values: None,
        individual: None,
        rationale: "schema.org has no type for a rule an AI system must \
                    follow. schema:DefinedTerm is the nearest parent: a \
                    published rule is a coded entry (termCode BAGS-01) in a \
                    published set (inDefinedTermSet), which is how the \
                    canonical schema (nyuchi/data-infra) models it. \
                    schema:CreativeWork was the first choice and was \
                    dropped to match that model; the standard document \
                    itself is the CreativeWork. The subclass adds what a \
                    DefinedTerm lacks: that the term binds an AI system, \
                    with a severity and a scope.",
    },
    Term {
        id: "PlatformGuardrail",
        kind: Kind::Class,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "Platform Guardrail",
        comment: "A guardrail set by the platform operator that applies to \
                  every AI surface it names. Only the operator can change \
                  one. A core platform guardrail applies to every surface \
                  whatever it names, and can never be deleted.",
        sub_class_of: Some("bundu:AIGuardrail"),
        domain_includes: None,
        range: None,
        values: None,
        individual: None,
        rationale: "The standard has two tiers of rule with different \
                    authority, and a consumer composing a system prompt must \
                    know which tier a rule belongs to: a platform rule is \
                    non-negotiable, an entity rule is additional. A property \
                    on AIGuardrail could carry the tier, but the tiers \
                    differ in who may write them and what they may do, \
                    which is a difference of kind.",
    },
    Term {
        id: "EntityGuardrail",
        kind: Kind::Class,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "Entity Guardrail",
        comment: "A guardrail an organisation, business or app tenant adds \
                  for its own AI surfaces. Entity guardrails stack on \
                  platform guardrails: they may add restrictions or tone \
                  guidance, and can never disable, override or contradict a \
                  platform guardrail. Where two rules conflict, the \
                  stricter one wins.",
        sub_class_of: Some("bundu:AIGuardrail"),
        domain_includes: None,
        range: None,
        values: None,
        individual: None,
        rationale: "The counterpart of PlatformGuardrail. Typing it \
                    separately is what lets a consumer enforce the \
                    standard's stacking rule mechanically: a rule typed \
                    EntityGuardrail can only ever narrow what a \
                    PlatformGuardrail allows.",
    },
    Term {
        id: "GuardrailCategory",
        kind: Kind::Class,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "Guardrail Category",
        comment: "The kind of harm or conduct a guardrail addresses, such \
                  as child safety or prompt injection, when it is published \
                  as a term of its own with a name and description. Its \
                  schema:termCode is the key that guardrailCategory \
                  carries.",
        sub_class_of: Some("schema:DefinedTerm"),
        domain_includes: None,
        range: None,
        values: None,
        individual: None,
        rationale: "Categories are a controlled set, so each one is a \
                    schema:DefinedTerm in a DefinedTermSet that the \
                    standard publishes. This class marks a DefinedTerm as a \
                    guardrail category, so a consumer can tell it from any \
                    other term. The category keys themselves are the \
                    published values of guardrailCategory.",
    },
    Term {
        id: "guardrailSeverity",
        kind: Kind::Property,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "guardrail severity",
        comment: "How serious a breach of a guardrail is, and therefore how \
                  firmly the system must hold to it.",
        sub_class_of: None,
        domain_includes: Some("bundu:AIGuardrail"),
        range: Some("schema:Text"),
        values: Some(GUARDRAIL_SEVERITY_VALUES),
        individual: None,
        rationale: "schema.org has no severity property for a rule. The \
                    values are the ones the live guardrail store already \
                    validates, so the vocabulary matches the data rather \
                    than inventing a scale. Published as plain values, like \
                    recognitionBasis, so the spelling is never guessed.",
    },
    Term {
        id: "guardrailCategory",
        kind: Kind::Property,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "guardrail category",
        comment: "The kind of harm or conduct a guardrail addresses, as a \
                  key from the Bundu AI Guardrails Standard. An entity \
                  guardrail uses the same keys.",
        sub_class_of: None,
        domain_includes: Some("bundu:AIGuardrail"),
        range: Some("schema:Text"),
        values: Some(GUARDRAIL_CATEGORY_VALUES),
        individual: None,
        rationale: "schema:category takes free text or a Thing, so two \
                    systems would spell the same category differently and \
                    could not compare their rules. Publishing the keys as \
                    the values of one property, as recognitionBasis does, \
                    fixes the spelling. The keys are the ones in BAGS \
                    1.0.0, owner-approved on 2026-10-04.",
    },
    Term {
        id: "BAGS",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "Bundu AI Guardrails Standard (BAGS)",
        comment: "The set of platform rules every AI surface in the Bundu \
                  ecosystem applies, each with a stable code from BAGS-01. \
                  Edition 1.0.0 has twelve core rules. The standard is \
                  published under CC BY 4.0.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: &["schema:DefinedTermSet"],
            term_code: None,
            in_set: None,
            facts: &[("bundu:standardVersion", Literal::Text("1.0.0"))],
        }),
        rationale: "Each rule names the set it belongs to with \
                    schema:inDefinedTermSet, which needs a URI for the set. \
                    The standard's web page will move and be redesigned; \
                    this URI will not, so rules, audits and guardrail \
                    stores can point at it permanently.",
    },
    Term {
        id: "BAGS-01",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-01: Child safety and minors",
        comment: "Rule BAGS-01 of the Bundu AI Guardrails Standard 1.0.0: \
                  child safety and minors. A core platform guardrail: \
                  category child_safety, severity critical. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-01"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("child_safety")),
                ("bundu:guardrailSeverity", Literal::Text("critical")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "BAGS-02",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-02: Self-harm and suicide",
        comment: "Rule BAGS-02 of the Bundu AI Guardrails Standard 1.0.0: \
                  self-harm and suicide. A core platform guardrail: \
                  category self_harm, severity critical. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-02"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("self_harm")),
                ("bundu:guardrailSeverity", Literal::Text("critical")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "BAGS-03",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-03: Violent extremism and terrorism",
        comment: "Rule BAGS-03 of the Bundu AI Guardrails Standard 1.0.0: \
                  violent extremism and terrorism. A core platform guardrail: \
                  category violent_extremism, severity critical. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-03"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("violent_extremism")),
                ("bundu:guardrailSeverity", Literal::Text("critical")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "BAGS-04",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-04: Hate and harassment",
        comment: "Rule BAGS-04 of the Bundu AI Guardrails Standard 1.0.0: \
                  hate and harassment. A core platform guardrail: \
                  category hate_harassment, severity high. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-04"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("hate_harassment")),
                ("bundu:guardrailSeverity", Literal::Text("high")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "BAGS-05",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-05: Sexual content",
        comment: "Rule BAGS-05 of the Bundu AI Guardrails Standard 1.0.0: \
                  sexual content. A core platform guardrail: \
                  category sexual_content, severity high. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-05"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("sexual_content")),
                ("bundu:guardrailSeverity", Literal::Text("high")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "BAGS-06",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-06: Weapons and illegal activity",
        comment: "Rule BAGS-06 of the Bundu AI Guardrails Standard 1.0.0: \
                  weapons and illegal activity. A core platform guardrail: \
                  category weapons_illegal, severity critical. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-06"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("weapons_illegal")),
                ("bundu:guardrailSeverity", Literal::Text("critical")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "BAGS-07",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-07: Privacy and personal data",
        comment: "Rule BAGS-07 of the Bundu AI Guardrails Standard 1.0.0: \
                  privacy and personal data. A core platform guardrail: \
                  category privacy, severity critical. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-07"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("privacy")),
                ("bundu:guardrailSeverity", Literal::Text("critical")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "BAGS-08",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-08: Prompt injection and jailbreak resistance",
        comment: "Rule BAGS-08 of the Bundu AI Guardrails Standard 1.0.0: \
                  prompt injection and jailbreak resistance. A core platform guardrail: \
                  category prompt_injection, severity critical. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-08"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("prompt_injection")),
                ("bundu:guardrailSeverity", Literal::Text("critical")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "BAGS-09",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-09: Impersonation and misinformation",
        comment: "Rule BAGS-09 of the Bundu AI Guardrails Standard 1.0.0: \
                  impersonation and misinformation. A core platform guardrail: \
                  category impersonation_misinformation, severity high. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-09"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("impersonation_misinformation")),
                ("bundu:guardrailSeverity", Literal::Text("high")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "BAGS-10",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-10: Medical, legal and financial advice",
        comment: "Rule BAGS-10 of the Bundu AI Guardrails Standard 1.0.0: \
                  medical, legal and financial advice. A core platform guardrail: \
                  category professional_advice, severity high. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-10"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("professional_advice")),
                ("bundu:guardrailSeverity", Literal::Text("high")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "BAGS-11",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-11: Ubuntu, cultural respect and African-language dignity",
        comment: "Rule BAGS-11 of the Bundu AI Guardrails Standard 1.0.0: \
                  Ubuntu, cultural respect and African-language dignity. A core platform guardrail: \
                  category cultural_respect, severity high. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-11"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("cultural_respect")),
                ("bundu:guardrailSeverity", Literal::Text("high")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "BAGS-12",
        kind: Kind::Individual,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "BAGS-12: Honest refusal",
        comment: "Rule BAGS-12 of the Bundu AI Guardrails Standard 1.0.0: \
                  honest refusal. A core platform guardrail: \
                  category refusal_style, severity medium. The rule's wording is \
                  published in the standard.",
        sub_class_of: None,
        domain_includes: None,
        range: None,
        values: None,
        individual: Some(Individual {
            types: BAGS_RULE_TYPES,
            term_code: Some("BAGS-12"),
            in_set: Some("bundu:BAGS"),
            facts: &[
                ("bundu:guardrailCategory", Literal::Text("refusal_style")),
                ("bundu:guardrailSeverity", Literal::Text("medium")),
                ("bundu:isCoreGuardrail", Literal::Boolean(true)),
                ("bundu:standardVersion", Literal::Text("1.0.0")),
            ],
        }),
        rationale: BAGS_RULE_RATIONALE,
    },
    Term {
        id: "appliesToSurface",
        kind: Kind::Property,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "applies to surface",
        comment: "An AI surface a guardrail applies to: a named product or \
                  assistant, such as nyuchi-ai. Repeatable. A guardrail \
                  with no surface applies to every surface, and a core \
                  platform guardrail applies to every surface whatever it \
                  names.",
        sub_class_of: None,
        domain_includes: Some("bundu:AIGuardrail"),
        range: Some("schema:Text"),
        values: None,
        individual: None,
        rationale: "schema:audience says who a work is for, and \
                    schema:applicationCategory classifies software; neither \
                    names the deployed AI surface a rule binds. Surfaces \
                    are what the guardrail service filters on when it \
                    compiles the rules for one surface, so the scope has to \
                    be data, not prose.",
    },
    Term {
        id: "enforcementLayer",
        kind: Kind::Property,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "enforcement layer",
        comment: "Where in an AI system a guardrail is enforced: for \
                  example in the system prompt, on the input before the \
                  model sees it, or on the output before a person sees \
                  it. Repeatable. The values are fixed by the Bundu AI \
                  Guardrail Standard.",
        sub_class_of: None,
        domain_includes: Some("bundu:AIGuardrail"),
        range: Some("schema:Text"),
        values: None,
        individual: None,
        rationale: "A rule enforced only by prompt guidance is weaker than \
                    one also checked on the output, and an auditor needs to \
                    see which a system claims. schema.org has nothing for \
                    this. The values are left open until the standard \
                    publishes them, rather than guessing a list that would \
                    then be permanent.",
    },
    Term {
        id: "isCoreGuardrail",
        kind: Kind::Property,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "is core guardrail",
        comment: "Whether a platform guardrail is core. A core guardrail \
                  applies to every AI surface whatever its surfaces say, \
                  cannot be disabled by any app or entity, and can never be \
                  deleted.",
        sub_class_of: None,
        domain_includes: Some("bundu:PlatformGuardrail"),
        range: Some("schema:Boolean"),
        values: None,
        individual: None,
        rationale: "Core status changes how every other field of the rule is \
                    read: a core rule ignores appliesToSurface. A consumer \
                    that missed it would let a surface opt out of a rule \
                    nobody may opt out of. schema.org has no equivalent.",
    },
    Term {
        id: "standardVersion",
        kind: Kind::Property,
        domain: Domain::AiGuardrails,
        status: Status::Stable,
        label: "standard version",
        comment: "The version of a Bundu standard that a rule, policy or \
                  implementation was written against, such as the version \
                  of the Bundu AI Guardrails Standard a guardrail conforms \
                  to.",
        sub_class_of: None,
        domain_includes: Some("schema:CreativeWork"),
        range: Some("schema:Text"),
        values: None,
        individual: None,
        rationale: "schema:version is the version of the thing itself, so a \
                    guardrail at its own version 3 cannot also say it was \
                    written against version 1.2 of the standard. \
                    dct:conformsTo needs a URI for each version of the \
                    standard, which the Bundu standards do not mint yet. \
                    Named generically because every Bundu standard will \
                    need it, not only the guardrail standard.",
    },
];

/// Look a term up by its exact id.
pub fn find(id: &str) -> Option<&'static Term> {
    TERMS.iter().find(|t| t.id == id)
}

/// Terms in a given subject area, in declaration order.
pub fn in_domain(domain: Domain) -> impl Iterator<Item = &'static Term> {
    TERMS.iter().filter(move |t| t.domain == domain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// A term id is the last segment of a permanent public URI. Anything
    /// needing escaping, or containing a path separator, would either break
    /// the URI or smuggle a taxonomy into it.
    #[test]
    fn ids_are_flat_and_uri_safe() {
        for t in TERMS {
            assert!(!t.id.is_empty(), "empty id");
            assert!(
                !t.id.contains('/'),
                "{}: ids must be flat — subject areas group the index, never the URI",
                t.id
            );
            // Hyphens are allowed between alphanumerics, so a rule keeps its
            // published code (BAGS-01) as its id. A hyphen is unreserved in a
            // URI and never needs escaping.
            assert!(
                t.id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
                    && !t.id.starts_with('-')
                    && !t.id.ends_with('-')
                    && !t.id.contains("--"),
                "{}: ids must be ASCII alphanumerics, with single inner hyphens at \
                 most, so the URI never needs escaping",
                t.id
            );
        }
    }

    #[test]
    fn ids_are_unique() {
        let mut seen = HashSet::new();
        for t in TERMS {
            assert!(seen.insert(t.id), "duplicate term id: {}", t.id);
        }
    }

    /// The README sets this bar explicitly: a term without a defensible
    /// rationale does not belong. Enforcing it here stops the vocabulary
    /// growing by accretion.
    #[test]
    fn every_term_justifies_itself() {
        for t in TERMS {
            assert!(
                t.rationale.len() > 80,
                "{}: rationale must explain what was tried first and why \
                 schema.org did not fit",
                t.id
            );
            assert!(!t.comment.is_empty(), "{}: missing definition", t.id);
            assert!(!t.label.is_empty(), "{}: missing label", t.id);
        }
    }

    #[test]
    fn classes_and_properties_carry_the_right_fields() {
        for t in TERMS {
            match t.kind {
                Kind::Class => {
                    assert!(t.individual.is_none(), "{}: only individuals", t.id);
                    assert!(
                        t.sub_class_of.is_some(),
                        "{}: a class must name its schema.org parent, or consumers \
                         that do not know this vocabulary read nothing at all",
                        t.id
                    );
                    assert!(t.range.is_none(), "{}: classes have no range", t.id);
                }
                Kind::Property => {
                    assert!(
                        t.domain_includes.is_some() && t.range.is_some(),
                        "{}: a property must say what it applies to and what its \
                         value is",
                        t.id
                    );
                    assert!(
                        t.sub_class_of.is_none(),
                        "{}: properties are not subclasses",
                        t.id
                    );
                }
                Kind::Individual => {
                    let i = t
                        .individual
                        .unwrap_or_else(|| panic!("{}: an individual must say what it is", t.id));
                    assert!(
                        i.types.iter().any(|ty| ty.starts_with("schema:")),
                        "{}: an individual needs a schema.org type, or consumers \
                         that do not know this vocabulary read nothing at all",
                        t.id
                    );
                    assert!(
                        t.sub_class_of.is_none()
                            && t.domain_includes.is_none()
                            && t.range.is_none(),
                        "{}: individuals are neither classes nor properties",
                        t.id
                    );
                }
            }
        }
    }

    /// Only properties enumerate, and an enumeration with an undocumented
    /// value is worse than none — the implementer guesses a spelling.
    #[test]
    fn enumerations_are_documented_and_unique() {
        for t in TERMS {
            let Some(values) = t.values else { continue };
            assert_eq!(
                t.kind,
                Kind::Property,
                "{}: only properties enumerate",
                t.id
            );
            assert!(!values.is_empty(), "{}: empty enumeration", t.id);

            let mut seen = HashSet::new();
            for (value, description) in values {
                assert!(seen.insert(value), "{}: duplicate value {value}", t.id);
                assert!(
                    !description.is_empty(),
                    "{}: value {value} has no description",
                    t.id
                );
                assert!(
                    value.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                    "{}: value {value} should be lower_snake_case",
                    t.id
                );
            }
        }
    }

    /// `Domain::ALL` drives the index. A domain missing from it would hide
    /// every term in that subject area from the only page listing them.
    #[test]
    fn every_domain_appears_in_the_index() {
        for t in TERMS {
            assert!(
                Domain::ALL.contains(&t.domain),
                "{}: subject area {:?} is missing from Domain::ALL, so the term \
                 would not appear on the index page",
                t.id,
                t.domain
            );
        }
    }

    /// A class may extend another class here, but the chain has to end at
    /// schema.org. Otherwise a consumer that does not know this vocabulary
    /// reads nothing at all.
    #[test]
    fn every_class_chain_reaches_schema_org() {
        for t in TERMS.iter().filter(|t| t.kind == Kind::Class) {
            let mut parent = t.sub_class_of.expect("classes name a parent");
            let mut hops = 0;
            while let Some(local) = parent.strip_prefix("bundu:") {
                let p = find(local)
                    .unwrap_or_else(|| panic!("{}: parent bundu:{local} is not a term", t.id));
                assert_eq!(
                    p.kind,
                    Kind::Class,
                    "{}: parent {local} is not a class",
                    t.id
                );
                parent = p.sub_class_of.expect("classes name a parent");
                hops += 1;
                assert!(hops < 8, "{}: subclass cycle", t.id);
            }
            assert!(
                parent.starts_with("schema:"),
                "{}: parent chain ends at {parent}, not schema.org",
                t.id
            );
        }
    }

    /// Every `bundu:` reference in a definition must resolve here, or the
    /// published JSON-LD points at a URI that 404s.
    #[test]
    fn no_dangling_bundu_references() {
        for t in TERMS {
            let mut refs: Vec<&str> = [t.sub_class_of, t.domain_includes, t.range]
                .into_iter()
                .flatten()
                .collect();
            if let Some(i) = t.individual {
                refs.extend(i.types);
                refs.extend(i.in_set);
                refs.extend(i.facts.iter().map(|(p, _)| *p));
            }
            for r in refs {
                if let Some(local) = r.strip_prefix("bundu:") {
                    assert!(
                        find(local).is_some(),
                        "{}: bundu:{local} is not a term",
                        t.id
                    );
                }
                assert!(
                    r.starts_with("bundu:") || r.starts_with("schema:"),
                    "{}: {r} uses a prefix the published @context does not declare",
                    t.id
                );
            }
        }
    }

    /// The guardrail terms were pending while the Bundu AI Guardrails
    /// Standard was being written. BAGS 1.0.0 was published on bundu.org on
    /// 2026-10-04 (bundu-labs/marketing#123), so they are stable: their
    /// meaning is now fixed.
    #[test]
    fn guardrail_terms_are_stable_now_the_standard_is_published() {
        for t in in_domain(Domain::AiGuardrails) {
            assert_eq!(t.status, Status::Stable, "{}", t.id);
        }
        assert_eq!(Status::Pending.vs_term_status(), "testing");
        assert_eq!(Status::Stable.vs_term_status(), "stable");
    }

    #[test]
    fn the_canonical_schema_and_guardrail_terms_are_present() {
        for id in [
            // Canonical schema (nyuchi/data-infra) and the Nyuchi API.
            "PublishingHouse",
            "NewsDesk",
            "defaultFamily",
            "assuranceLevel",
            "attestation",
            "kAnonymity",
            "maxReidentificationRisk",
            // Bundu AI Guardrails Standard.
            "AIGuardrail",
            "PlatformGuardrail",
            "EntityGuardrail",
            "GuardrailCategory",
            "guardrailSeverity",
            "appliesToSurface",
            "enforcementLayer",
            "isCoreGuardrail",
            "standardVersion",
            "guardrailCategory",
            "BAGS",
        ] {
            assert!(find(id).is_some(), "missing term: {id}");
        }
    }

    /// BAGS 1.0.0 codes are final (owner-approved 2026-10-04). Every rule
    /// keeps its code as its id, sits in the BAGS set, and states only
    /// values the vocabulary publishes.
    #[test]
    fn bags_rules_are_complete_and_consistent() {
        let allowed = |prop: &str| -> Vec<&'static str> {
            find(prop)
                .and_then(|t| t.values)
                .expect("enumerated")
                .iter()
                .map(|(v, _)| *v)
                .collect()
        };
        let categories = allowed("guardrailCategory");
        let severities = allowed("guardrailSeverity");
        let mut seen_categories = Vec::new();
        for n in 1..=12 {
            let code = format!("BAGS-{n:02}");
            let t = find(&code).unwrap_or_else(|| panic!("missing rule {code}"));
            assert_eq!(t.kind, Kind::Individual);
            let i = t.individual.expect("individual");
            assert_eq!(i.term_code, Some(t.id), "{code}: id and termCode differ");
            assert_eq!(i.in_set, Some("bundu:BAGS"));
            assert!(i.types.contains(&"bundu:PlatformGuardrail"));
            for (prop, value) in i.facts {
                if let Literal::Text(v) = value {
                    match *prop {
                        "bundu:guardrailCategory" => {
                            assert!(categories.contains(v), "{code}: unknown category {v}");
                            seen_categories.push(*v);
                        }
                        "bundu:guardrailSeverity" => {
                            assert!(severities.contains(v), "{code}: unknown severity {v}")
                        }
                        _ => {}
                    }
                }
            }
        }
        assert!(find("BAGS-13").is_none(), "BAGS 1.0.0 has twelve rules");
        seen_categories.sort_unstable();
        let mut all = categories.clone();
        all.sort_unstable();
        assert_eq!(
            seen_categories, all,
            "each category has exactly one 1.0.0 rule"
        );
    }

    #[test]
    fn guardrail_severity_matches_the_live_store() {
        // The values the shamwari.guardrails validator accepts. Drifting from
        // them would publish a spelling no stored rule uses.
        let values: Vec<_> = find("guardrailSeverity")
            .and_then(|t| t.values)
            .expect("enumerated")
            .iter()
            .map(|(v, _)| *v)
            .collect();
        assert_eq!(values, ["critical", "high", "medium", "low"]);
    }

    #[test]
    fn every_term_is_findable() {
        for t in TERMS {
            assert!(find(t.id).is_some(), "{} not resolvable via find()", t.id);
        }
        assert!(find("NotATerm").is_none());
        assert!(find("family").is_none(), "lookup must be case-sensitive");
    }

    #[test]
    fn in_domain_partitions_the_vocabulary() {
        let counted: usize = Domain::ALL.iter().map(|d| in_domain(*d).count()).sum();
        assert_eq!(
            counted,
            TERMS.len(),
            "every term must belong to exactly one subject area"
        );
    }

    #[test]
    fn recognition_basis_covers_the_researched_cases() {
        // Each value exists because a real jurisdiction produced it. Dropping
        // one silently reintroduces the bug the term was added to fix:
        // an unregistered-but-real entity looking like a registered one with
        // missing paperwork.
        let t = find("recognitionBasis").expect("term present");
        let values: Vec<_> = t
            .values
            .expect("enumerated")
            .iter()
            .map(|(v, _)| *v)
            .collect();
        for expected in [
            "customary",      // Zimbabwean communal land under a village head
            "statutory",      // Indonesian desa adat
            "registry",       // Mexican ejido
            "member_consent", // South African stokvel
            "contested",      // overlapping Zimbabwean Acts
            "none",
        ] {
            assert!(values.contains(&expected), "missing value: {expected}");
        }
    }
}
