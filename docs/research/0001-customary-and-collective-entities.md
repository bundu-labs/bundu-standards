# Research note 0001 — customary and collective entities

**Question:** the Bundu entity model classifies entities as `family`,
`organization`, `community` or `place_owner`. Do the collective and customary
structures that dominate African, Latin American and South-East Asian economic
life need additional entity *types*, or is something else missing?

**Answer:** almost none of them need a new type. What is missing is a way to
record **why an entity is legitimate**, because that varies enormously for
structures that are otherwise identical in shape — and it determines what
verification evidence is even meaningful to ask for.

---

## What the sources show

### The scale is not marginal

South Africa has roughly **800,000 stokvels** holding over **R50 billion**,
with an estimated **11 million participants** — about one in five South
Africans.[^stokvel-scale] By 1992, Mexican **ejidos** held more than **two
thirds of the country's arable land and forest**, the most extensive
state-managed land tenure transformation in modern Latin American
history.[^ejido-scale]

These are not edge cases to be handled after the main model works. In their
own markets they *are* the main model.

The same structure recurs under different names: stokvel in South Africa,
*chama* in Kenya, *esusu* in Nigeria, *susu* in Ghana — different words,
materially the same practice.[^rosca-names]

### Legal recognition varies wildly for structurally identical things

This is the finding that matters.

| Structure | Jurisdiction | Legal standing |
|---|---|---|
| Stokvel | South Africa | Defined in statute (National Credit Act 2005) but **not a juristic person** under it. Some fund managers require registration as a club or association before they can invest.[^stokvel-legal] |
| *Desa adat* | Indonesia | **Public legal entity**, equal standing with administrative villages under the 2014 Village Law, conditional on the customary community continuing to exist.[^desa-adat] |
| Ejido | Mexico | **Registered** — the National Agrarian Registry documents land tenure and community rights.[^ejido-registry] |
| Communal land under a village head | Zimbabwe | Traditional leaders hold **constitutionally recognised** land administration powers, but the Communal Land Act and Traditional Leaders Act overlap, producing genuine ambiguity about who holds authority.[^zim-leaders] |

Four collective land- or money-holding bodies. One has no legal personhood,
one has full statutory personhood, one has a registry entry, one has
constitutional recognition undercut by conflicting statutes.

**A model that treats "verified" as a single ladder cannot express this.** The
evidence that proves an ejido is real — a registry number — does not exist for
a stokvel, and demanding it is a category error rather than a failed check.

### The global standard admits the gap

ISO 20275 (Entity Legal Forms), maintained by GLEIF, lists **over 3,400 legal
forms across 185+ jurisdictions**, and GLEIF states plainly that identifying
further forms "remains a work in progress", inviting submissions to
`elf@gleif.org`.[^elf]

Two things follow. The gap is real and acknowledged by the people who maintain
the world's entity-form registry. And the enumerate-every-local-form strategy
scales to 3,400 entries and still is not finished — which is a warning, not a
model to copy.

---

## What this means for the vocabulary

### Do not add per-structure classes

The tempting move is `Stokvel`, `Chama`, `Ejido`, `DesaAdat`, `Barangay`.
Resist it. ISO 20275 shows where that path ends: thousands of jurisdiction-
specific entries, permanently incomplete, each needing maintenance by someone
who understands that jurisdiction.

It also encodes the wrong thing. A stokvel and a chama are the same structure
under two names; modelling them as different classes makes a Kenyan and a
South African savings group incomparable for no reason.

Local names belong in `legalName` or `alternateName`, not in the type system.

### The existing enum already covers the shapes

Checked against the researched structures:

- stokvel / chama / susu → `community`, with `bundu.informalEconomy.operatingModel: "rotating"` — **already in the schema**
- *desa adat* → `community`, or `place_owner` where territorial custody is the point
- ejido → `community` + `place_owner`
- Zimbabwean village under a headman → `community` + `place_owner`

`operatingModel` already carries `cooperative | individual | family_run |
community_managed | rotating | association`, and `registrationStatus` already
carries `unregistered | pending | tax_only | fully_registered`. Much of this
was anticipated.

### The actual gap: basis of recognition

Nothing records **what makes this entity legitimate**, and that is the field
that determines which verification evidence is meaningful:

- If recognition is **customary**, asking for a registration number is
  incoherent; attestation by a recognised authority is the evidence.
- If recognition is **statutory**, the statute is the evidence.
- If recognition is **registry-based**, the registry entry is the evidence.
- If recognition rests on **member consent alone**, the members are the
  evidence — and no external document will ever exist.

`verificationMethod` already has `community_attestation`,
`government_registry`, `honeycomb_attestation` and `in_person`, so the
*methods* are largely there. What is missing is the property that says which
method is appropriate, rather than leaving every unregistered entity looking
like a registered one with missing paperwork.

**Proposed term: `recognitionBasis`** — one property, values roughly
`customary | statutory | registry | member_consent | none`. It is orthogonal
to entity type, applies across all jurisdictions, and does not grow with the
number of countries served.

A second candidate, weaker and worth deferring until there is a concrete need:
a membership role for customary office. `membershipRole` currently has
`founder | admin | manager | representative | member | contributor | follower
| kin`, none of which describes a village head or chief holding an office
under customary law rather than a position in an organisation.

---

## What this challenges

The starting assumption was that these structures need to be added as entity
types because Western identity schemas have no word for them. The first half
is right — ISO 20275 concedes the gap, and schema.org has nothing for any of
them. The second half does not follow.

The structures are not exotic *shapes*. A rotating savings group is
recognisably a group with members and a purpose; the model already handles
that. What is exotic, from the perspective of a registry-based identity
system, is that **they can be entirely real and entirely unregistered at the
same time** — and that their legitimacy comes from somewhere a company
register cannot see.

Model the basis of legitimacy, not the local name. One property earns more
than fifty classes.

---

## Open questions

- **Does `recognitionBasis` belong on the entity, or on the verification
  event?** An entity's basis can change — a stokvel that registers as an
  association acquires a statutory basis without ceasing to be a stokvel.
  Recording it on `verifications` would preserve that history; recording it on
  the entity makes it queryable. Possibly both.
- **Should Bundu submit to GLEIF?** ISO 20275 explicitly invites additions.
  Getting African customary forms into the global standard would be more
  consequential than publishing a parallel vocabulary — and the two are not
  mutually exclusive.
- **Contested authority.** Zimbabwe's overlapping Communal Land Act and
  Traditional Leaders Act mean two bodies can each legitimately claim
  authority over the same land. No single-valued field expresses that.

---

[^stokvel-scale]: National Stokvel Association of South Africa figures, via [Barter McKellar, *Understanding Stokvels in South Africa: A Legal Perspective*](https://www.bartermckellar.law/banking-financial-law-explained/understanding-stokvels-in-south-africa-a-legal-perspective) and [Stokvel — Wikipedia](https://en.wikipedia.org/wiki/Stokvel).
[^ejido-scale]: [*The evolution and resilience of community-based land tenure in rural Mexico*, ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S0264837708000641).
[^rosca-names]: [Africa Social Work & Development Network, *Stokvels*](https://africasocialwork.net/stokvels-an-opportunity-for-development-at-family-community-and-national-level-or-not/).
[^stokvel-legal]: National Credit Act 34 of 2005 definition; juristic-person status discussed in [Barter McKellar](https://www.bartermckellar.law/banking-financial-law-explained/understanding-stokvels-in-south-africa-a-legal-perspective) and [Stokvel insurance — Mondaq](https://www.mondaq.com/southafrica/insurance-laws-and-products/670766/stokvel-insurance-a-potential-south-african-phenomenon).
[^desa-adat]: Indonesian Village Law No. 6/2014. See [Village Act — Wikipedia](https://en.wikipedia.org/wiki/Indonesian_Village_Law) and [*Traditional village institutions and the Village Law*, Inside Indonesia](https://www.insideindonesia.org/editions/edition-128-apr-jun-2017/traditional-village-institutions-and-the-village-law).
[^ejido-registry]: [FAO, *Indigenous peoples, land tenure and land policy in Latin America*](https://www.fao.org/4/y5407t/y5407t0a.htm).
[^zim-leaders]: [Traditional Leaders Act (Chapter 29:17), ZimLII](https://zimlii.org/akn/zw/act/1998/25/eng@2016-12-31); [Communal Land Act, ZimLII](https://zimlii.org/akn/zw/act/1982/20/eng@2016-12-31); [*The role of traditional leaders in Zimbabwe: are they still relevant?*, SciELO SA](https://scielo.org.za/scielo.php?script=sci_arttext&pid=S2077-49072016000100003).
[^elf]: [GLEIF, *ISO 20275: Entity Legal Forms Code List*](https://www.gleif.org/en/about-lei/iso-20275-entity-legal-forms-code-list).
