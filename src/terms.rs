//! The vocabulary.
//!
//! Every term here exists because schema.org has no word for something the
//! Bundu ecosystem needs to say. That test is deliberately strict: if
//! schema.org can express it, we use schema.org and add nothing. A
//! vocabulary earns trust by being small.
//!
//! Each term carries a `rationale` explaining what was tried first and why
//! it did not fit. Anyone auditing the model later can then judge whether
//! the term is still needed, rather than guessing at intent.

/// Whether a term is a class (a kind of thing) or a property (something a
/// thing has).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Class,
    Property,
}

impl Kind {
    /// The RDF type URI for this kind of term.
    pub const fn rdf_type(self) -> &'static str {
        match self {
            Kind::Class => "rdfs:Class",
            Kind::Property => "rdf:Property",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Kind::Class => "Class",
            Kind::Property => "Property",
        }
    }
}

/// A single vocabulary term.
pub struct Term {
    /// The term name, and the last path segment of its URI.
    pub id: &'static str,
    pub kind: Kind,
    pub label: &'static str,
    /// What the term means.
    pub comment: &'static str,
    /// Parent class, for classes.
    pub sub_class_of: Option<&'static str>,
    /// What a property attaches to.
    pub domain: Option<&'static str>,
    /// What a property's value is.
    pub range: Option<&'static str>,
    /// Why schema.org was insufficient. The justification for the term
    /// existing at all.
    pub rationale: &'static str,
}

/// The full vocabulary.
pub const TERMS: &[Term] = &[
    Term {
        id: "Family",
        kind: Kind::Class,
        label: "Family",
        comment: "A kinship group treated as a first-class entity: it holds a \
                  handle, a verification standing and members, exactly as an \
                  organization does. In the Bundu model a person cannot exist \
                  without belonging to one.",
        sub_class_of: Some("schema:Organization"),
        domain: None,
        range: None,
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
        label: "Community",
        comment: "A group organised around a shared place, practice or purpose \
                  that is neither a family nor a registered organization. \
                  Savings groups, burial societies, cooperatives and \
                  neighbourhood associations.",
        sub_class_of: Some("schema:Organization"),
        domain: None,
        range: None,
        rationale: "Communities in this ecosystem are frequently unregistered \
                    and derive standing from collective attestation rather than \
                    from a registry. schema:Organization implies a formality \
                    most of them do not have and should not be required to \
                    perform in order to be represented.",
    },
    Term {
        id: "PlaceOwner",
        kind: Kind::Class,
        label: "Place Owner",
        comment: "An entity whose relationship to the ecosystem is custody of a \
                  physical place — the party answerable for a venue, market \
                  stall, farm or landmark.",
        sub_class_of: Some("schema:Organization"),
        domain: None,
        range: None,
        rationale: "schema.org models places and organizations, but not the \
                    custodial relationship between them as a kind of entity. \
                    Custody matters here because it is what a verification \
                    claim attaches to when a place is claimed.",
    },
    Term {
        id: "verificationTier",
        kind: Kind::Property,
        label: "verification tier",
        comment: "How far an entity has progressed through verification. \
                  0 = unclaimed, 1 = claimed, 2 = verified, 3 = fully verified.",
        sub_class_of: None,
        domain: Some("schema:Thing"),
        range: Some("schema:Integer"),
        rationale: "An ordered ladder, not a boolean. schema.org offers no \
                    graduated trust signal, and a plain verified/unverified flag \
                    cannot express the difference between an entity someone has \
                    claimed and one whose registration documents have been \
                    reviewed.",
    },
    Term {
        id: "ubuntuScore",
        kind: Kind::Property,
        label: "ubuntu score",
        comment: "A standing derived from an entity's conduct toward others in \
                  the ecosystem, rather than from customer satisfaction.",
        sub_class_of: None,
        domain: Some("schema:Organization"),
        range: Some("schema:Number"),
        rationale: "schema:aggregateRating measures how well something served \
                    the person rating it. This measures something different — \
                    reciprocity and contribution — and conflating the two would \
                    let a highly rated but extractive entity appear trustworthy.",
    },
    Term {
        id: "communityVouches",
        kind: Kind::Property,
        label: "community vouches",
        comment: "The number of entities that have vouched for this one, where \
                  vouching carries reputational consequence for the voucher.",
        sub_class_of: None,
        domain: Some("schema:Organization"),
        range: Some("schema:Integer"),
        rationale: "Distinct from schema:reviewCount because a vouch is not a \
                    review: the voucher stakes their own standing on the claim. \
                    In economies without reliable registries this is often the \
                    only trust signal available, and it must not be read as \
                    popularity.",
    },
    Term {
        id: "isInformal",
        kind: Kind::Property,
        label: "is informal",
        comment: "Whether an entity operates in the informal economy — trading \
                  without formal registration.",
        sub_class_of: None,
        domain: Some("schema:Organization"),
        range: Some("schema:Boolean"),
        rationale: "Most economic activity in the ecosystem's markets is \
                    informal. Recording this plainly, as a neutral fact rather \
                    than an absence of data, is what lets informal traders be \
                    represented at all instead of appearing as incomplete \
                    records of formal ones.",
    },
];

/// Look a term up by its exact id.
pub fn find(id: &str) -> Option<&'static Term> {
    TERMS.iter().find(|t| t.id == id)
}

/// Terms of a given kind, in declaration order.
pub fn of_kind(kind: Kind) -> impl Iterator<Item = &'static Term> {
    TERMS.iter().filter(move |t| t.kind == kind)
}
