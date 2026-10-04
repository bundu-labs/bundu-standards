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
}

impl Domain {
    pub const fn label(self) -> &'static str {
        match self {
            Domain::IdentityAndKinship => "Identity and kinship",
            Domain::TrustAndRecognition => "Trust and recognition",
            Domain::InformalEconomy => "Informal economy",
        }
    }

    /// Declaration order controls index order.
    pub const ALL: &'static [Domain] = &[
        Domain::IdentityAndKinship,
        Domain::TrustAndRecognition,
        Domain::InformalEconomy,
    ];
}

/// A single vocabulary term.
pub struct Term {
    /// The term name, and the last path segment of its URI.
    pub id: &'static str,
    pub kind: Kind,
    pub domain: Domain,
    pub label: &'static str,
    /// What the term means.
    pub comment: &'static str,
    /// Parent class, for classes.
    pub sub_class_of: Option<&'static str>,
    /// What a property attaches to.
    pub domain_includes: Option<&'static str>,
    /// What a property's value is.
    pub range: Option<&'static str>,
    /// Permitted values, where the property is an enumeration.
    pub values: Option<&'static [(&'static str, &'static str)]>,
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

/// The full vocabulary.
pub const TERMS: &[Term] = &[
    Term {
        id: "Family",
        kind: Kind::Class,
        domain: Domain::IdentityAndKinship,
        label: "Family",
        comment: "A kinship group treated as a first-class entity: it holds a \
                  handle, a verification standing and members, exactly as an \
                  organization does. In the Bundu model a person cannot exist \
                  without belonging to one.",
        sub_class_of: Some("schema:Organization"),
        domain_includes: None,
        range: None,
        values: None,
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
        label: "Community",
        comment: "A group organised around a shared place, practice or purpose \
                  that is neither a family nor a registered organization. \
                  Savings groups, burial societies, cooperatives and \
                  neighbourhood associations.",
        sub_class_of: Some("schema:Organization"),
        domain_includes: None,
        range: None,
        values: None,
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
        label: "Place Owner",
        comment: "An entity whose relationship to the ecosystem is custody of a \
                  physical place — the party answerable for a venue, market \
                  stall, farm or landmark.",
        sub_class_of: Some("schema:Organization"),
        domain_includes: None,
        range: None,
        values: None,
        rationale: "schema.org models places and organizations, but not the \
                    custodial relationship between them as a kind of entity. \
                    Custody matters here because it is what a verification \
                    claim attaches to when a place is claimed.",
    },
    Term {
        id: "recognitionBasis",
        kind: Kind::Property,
        domain: Domain::TrustAndRecognition,
        label: "recognition basis",
        comment: "What makes an entity legitimate, and therefore what evidence \
                  is meaningful when verifying it.",
        sub_class_of: None,
        domain_includes: Some("schema:Organization"),
        range: Some("schema:Text"),
        values: Some(RECOGNITION_BASIS_VALUES),
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
        label: "verification tier",
        comment: "How far an entity has progressed through verification, \
                  as an integer from 0 to 4: 0 = unverified, 1 = community \
                  (vouched for by people who know it), 2 = contact verified \
                  (a one-time passcode), 3 = government (registration \
                  documents reviewed), 4 = licensed (a sector licence \
                  confirmed). Published on Mukoko Kweli profiles and on their \
                  ActivityPub actors.",
        sub_class_of: None,
        domain_includes: Some("schema:Thing"),
        range: Some("schema:Integer"),
        values: None,
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
        label: "ubuntu score",
        comment: "A standing derived from an entity's conduct toward others in \
                  the ecosystem, rather than from customer satisfaction.",
        sub_class_of: None,
        domain_includes: Some("schema:Organization"),
        range: Some("schema:Number"),
        values: None,
        rationale: "schema:aggregateRating measures how well something served \
                    the person rating it. This measures something different — \
                    reciprocity and contribution — and conflating the two would \
                    let a highly rated but extractive entity appear trustworthy.",
    },
    Term {
        id: "communityVouches",
        kind: Kind::Property,
        domain: Domain::TrustAndRecognition,
        label: "community vouches",
        comment: "The number of entities that have vouched for this one, where \
                  vouching carries reputational consequence for the voucher.",
        sub_class_of: None,
        domain_includes: Some("schema:Organization"),
        range: Some("schema:Integer"),
        values: None,
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
        label: "is informal",
        comment: "Whether an entity operates in the informal economy — trading \
                  without formal registration.",
        sub_class_of: None,
        domain_includes: Some("schema:Organization"),
        range: Some("schema:Boolean"),
        values: None,
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
            assert!(
                t.id.chars().all(|c| c.is_ascii_alphanumeric()),
                "{}: ids must be plain ASCII alphanumerics so the URI never needs escaping",
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
