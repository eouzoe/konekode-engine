module

public import Mathlib.Data.Finset.Card
public import Mathlib.Data.Finset.Powerset
public import Mathlib.Data.Finset.Max

@[expose] public section

namespace Konekode

variable {α : Type*} [DecidableEq α]

structure LearningSpace (F : Set (Finset α)) : Prop where
  empty : ∅ ∈ F
  union : ∀ ⦃K L : Finset α⦄, K ∈ F → L ∈ F → K ∪ L ∈ F
  access : ∀ ⦃K : Finset α⦄, K ∈ F → K.Nonempty → ∃ q ∈ K, K.erase q ∈ F

def Outer (F : Set (Finset α)) (K : Finset α) (x : α) : Prop :=
  x ∉ K ∧ insert x K ∈ F

def Inner (F : Set (Finset α)) (K : Finset α) (x : α) : Prop :=
  x ∈ K ∧ K.erase x ∈ F

def Atom (F : Set (Finset α)) (q : α) (A : Finset α) : Prop :=
  A ∈ F ∧ q ∈ A ∧ ∀ B ∈ F, q ∈ B → B ⊆ A → B = A

inductive Reach (σ : α → Set (Finset α)) : Finset α → Prop
  | empty : Reach σ ∅
  | step {K C : Finset α} {q : α} : Reach σ K → C ∈ σ q → C ⊆ K → Reach σ (insert q K)

def canonical (F : Set (Finset α)) (q : α) : Set (Finset α) :=
  {C | ∃ A, Atom F q A ∧ C = A.erase q}

def proj (F : Set (Finset α)) (Q : Finset α) : Set (Finset α) :=
  {T | ∃ K ∈ F, T = K ∩ Q}

def AtomTrace (F : Set (Finset α)) (Q : Finset α) (q : α) (T : Finset α) : Prop :=
  ∃ A, Atom F q A ∧ T = A ∩ Q

variable {F : Set (Finset α)}

theorem insert_eq_union {K L : Finset α} {q : α} (hq : q ∈ L) (hsub : L.erase q ⊆ K) :
    insert q K = K ∪ L := by
  ext y
  simp only [Finset.mem_insert, Finset.mem_union]
  constructor
  · rintro (rfl | hy)
    · exact Or.inr hq
    · exact Or.inl hy
  · rintro (hy | hy)
    · exact Or.inr hy
    · by_cases hyq : y = q
      · exact Or.inl hyq
      · exact Or.inr (hsub (Finset.mem_erase.mpr ⟨hyq, hy⟩))

theorem augment_aux (h : LearningSpace F) :
    ∀ (n : ℕ) (L : Finset α), L.card = n → L ∈ F → ∀ K ∈ F, ¬ L ⊆ K →
      ∃ x ∈ L, x ∉ K ∧ insert x K ∈ F := by
  intro n
  induction n with
  | zero =>
    intro L hL _ K _ hLK
    rw [Finset.card_eq_zero] at hL
    subst hL
    exact absurd (Finset.empty_subset K) hLK
  | succ n ih =>
    intro L hL hLF K hKF hLK
    have hne : L.Nonempty := by
      rw [← Finset.card_pos, hL]
      omega
    obtain ⟨q, hqL, hq⟩ := h.access hLF hne
    by_cases hsub : L.erase q ⊆ K
    · have hqK : q ∉ K := by
        intro hqK
        apply hLK
        intro y hy
        by_cases hyq : y = q
        · rw [hyq]
          exact hqK
        · exact hsub (Finset.mem_erase.mpr ⟨hyq, hy⟩)
      refine ⟨q, hqL, hqK, ?_⟩
      rw [insert_eq_union hqL hsub]
      exact h.union hKF hLF
    · have hcard : (L.erase q).card = n := by
        rw [Finset.card_erase_of_mem hqL, hL]
        omega
      obtain ⟨x, hx, hxK, hins⟩ := ih (L.erase q) hcard hq K hKF hsub
      exact ⟨x, Finset.mem_of_mem_erase hx, hxK, hins⟩

theorem augment (h : LearningSpace F) {K L : Finset α} (hK : K ∈ F) (hL : L ∈ F)
    (hLK : ¬ L ⊆ K) : ∃ x ∈ L, x ∉ K ∧ insert x K ∈ F :=
  augment_aux h _ L rfl hL K hK hLK

theorem outer_determines (h : LearningSpace F) {K L : Finset α} (hK : K ∈ F) (hL : L ∈ F)
    (hout : ∀ x, Outer F K x ↔ Outer F L x) : K = L := by
  have key : ∀ {K L : Finset α}, K ∈ F → L ∈ F → (∀ x, Outer F K x ↔ Outer F L x) →
      L ⊆ K := by
    intro K L hK hL hout
    by_contra hLK
    have hnot : ¬ K ∪ L ⊆ K := fun hs => hLK (fun y hy => hs (Finset.mem_union_right K hy))
    obtain ⟨x, hx, hxK, hins⟩ := augment h hK (h.union hK hL) hnot
    have hxL : x ∈ L := by
      rcases Finset.mem_union.mp hx with hxK' | hxL
      · exact absurd hxK' hxK
      · exact hxL
    exact ((hout x).mp ⟨hxK, hins⟩).1 hxL
  exact Finset.Subset.antisymm (key hL hK (fun x => (hout x).symm)) (key hK hL hout)

theorem fringe_theorem (h : LearningSpace F) {K L : Finset α} (hK : K ∈ F) (hL : L ∈ F)
    (_ : ∀ x, Inner F K x ↔ Inner F L x) (hout : ∀ x, Outer F K x ↔ Outer F L x) : K = L :=
  outer_determines h hK hL hout

theorem exists_atom {K : Finset α} {q : α} (hK : K ∈ F) (hq : q ∈ K) :
    ∃ A, Atom F q A ∧ A ⊆ K := by
  classical
  let S := K.powerset.filter (fun B => B ∈ F ∧ q ∈ B)
  have hS : S.Nonempty :=
    ⟨K, Finset.mem_filter.mpr ⟨Finset.mem_powerset_self K, hK, hq⟩⟩
  obtain ⟨A, hA, hmin⟩ := Finset.exists_min_image S Finset.card hS
  obtain ⟨hAK', hAF, hqA⟩ := Finset.mem_filter.mp hA
  have hAK := Finset.mem_powerset.mp hAK'
  refine ⟨A, ⟨hAF, hqA, fun B hB hqB hBA => ?_⟩, hAK⟩
  have hBS : B ∈ S :=
    Finset.mem_filter.mpr ⟨Finset.mem_powerset.mpr (hBA.trans hAK), hB, hqB⟩
  exact Finset.eq_of_subset_of_card_le hBA (hmin B hBS)

theorem atom_one_item (h : LearningSpace F) {A : Finset α} {p q : α} (hp : Atom F p A)
    (hq : Atom F q A) : p = q := by
  obtain ⟨r, hrA, hr⟩ := h.access hp.1 ⟨p, hp.2.1⟩
  have key : ∀ {s : α}, Atom F s A → s = r := by
    intro s hs
    by_contra hsr
    have heq := hs.2.2 _ hr (Finset.mem_erase.mpr ⟨hsr, hs.2.1⟩) (Finset.erase_subset r A)
    have hnot : r ∉ A.erase r := by simp
    rw [heq] at hnot
    exact hnot hrA
  rw [key hp, key hq]

theorem reach_union (σ : α → Set (Finset α)) {K L : Finset α} (hK : Reach σ K)
    (hL : Reach σ L) : Reach σ (K ∪ L) := by
  induction hL with
  | empty => simpa using hK
  | step _ hC hCL ih =>
    rw [Finset.union_insert]
    exact Reach.step ih hC (hCL.trans Finset.subset_union_right)

theorem reach_access (σ : α → Set (Finset α)) {K : Finset α} (hK : Reach σ K) :
    K.Nonempty → ∃ q ∈ K, Reach σ (K.erase q) := by
  induction hK with
  | empty => exact fun hne => absurd hne Finset.not_nonempty_empty
  | @step K C q hK _ _ ih =>
    intro hne
    by_cases hq : q ∈ K
    · rw [Finset.insert_eq_of_mem hq] at hne ⊢
      exact ih hne
    · refine ⟨q, Finset.mem_insert_self q K, ?_⟩
      rw [Finset.erase_insert hq]
      exact hK

theorem rules_make_learning_space (σ : α → Set (Finset α)) :
    LearningSpace {K | Reach σ K} where
  empty := Reach.empty
  union := fun _ _ hK hL => reach_union σ hK hL
  access := fun _ hK hne => reach_access σ hK hne

theorem reach_canonical_of_mem (h : LearningSpace F) :
    ∀ (n : ℕ) (K : Finset α), K.card = n → K ∈ F → Reach (canonical F) K := by
  intro n
  induction n with
  | zero =>
    intro K hc _
    rw [Finset.card_eq_zero.mp hc]
    exact Reach.empty
  | succ n ih =>
    intro K hc hK
    have hne : K.Nonempty := by
      rw [← Finset.card_pos, hc]
      omega
    obtain ⟨q, hqK, hq⟩ := h.access hK hne
    obtain ⟨A, hA, hAK⟩ := exists_atom hK hqK
    have hprev := ih (K.erase q) (by rw [Finset.card_erase_of_mem hqK, hc]; omega) hq
    have hstep := Reach.step (q := q) hprev ⟨A, hA, rfl⟩ (Finset.erase_subset_erase q hAK)
    rwa [Finset.insert_erase hqK] at hstep

theorem canonical_presents (h : LearningSpace F) (K : Finset α) :
    Reach (canonical F) K ↔ K ∈ F := by
  constructor
  · intro hK
    induction hK with
    | empty => exact h.empty
    | @step K C q _ hC hCK ih =>
      obtain ⟨A, hA, rfl⟩ := hC
      rw [insert_eq_union hA.2.1 hCK]
      exact h.union ih hA.1
  · exact reach_canonical_of_mem h _ K rfl

theorem same_structure_iff (σ τ : α → Set (Finset α)) :
    {K | Reach σ K} = {K | Reach τ K} ↔
      canonical {K | Reach σ K} = canonical {K | Reach τ K} := by
  constructor
  · intro heq
    rw [heq]
  · intro heq
    ext K
    have hσ := canonical_presents (rules_make_learning_space σ) K
    have hτ := canonical_presents (rules_make_learning_space τ) K
    rw [heq] at hσ
    exact hσ.symm.trans hτ

theorem proj_learning_space (h : LearningSpace F) (Q : Finset α) :
    LearningSpace (proj F Q) where
  empty := ⟨∅, h.empty, by simp⟩
  union := by
    rintro _ _ ⟨K, hK, rfl⟩ ⟨L, hL, rfl⟩
    refine ⟨K ∪ L, h.union hK hL, ?_⟩
    ext y
    simp only [Finset.mem_union, Finset.mem_inter]
    tauto
  access := by
    classical
    intro T hT hne
    have hex : ∃ n, ∃ K ∈ F, K ∩ Q = T ∧ K.card = n := by
      obtain ⟨K, hK, rfl⟩ := hT
      exact ⟨K.card, K, hK, rfl, rfl⟩
    obtain ⟨K, hK, hKT, hc⟩ := Nat.find_spec hex
    have hKne : K.Nonempty := by
      obtain ⟨y, hy⟩ := hne
      rw [← hKT] at hy
      exact ⟨y, (Finset.mem_inter.mp hy).1⟩
    obtain ⟨r, hrK, hr⟩ := h.access hK hKne
    by_cases hrQ : r ∈ Q
    · refine ⟨r, ?_, K.erase r, hr, ?_⟩
      · rw [← hKT]
        exact Finset.mem_inter.mpr ⟨hrK, hrQ⟩
      · rw [← hKT]
        ext y
        simp only [Finset.mem_erase, Finset.mem_inter]
        tauto
    · exfalso
      have hsame : K.erase r ∩ Q = T := by
        rw [← hKT]
        ext y
        simp only [Finset.mem_inter, Finset.mem_erase]
        constructor
        · rintro ⟨⟨_, hy⟩, hyQ⟩
          exact ⟨hy, hyQ⟩
        · rintro ⟨hy, hyQ⟩
          refine ⟨⟨?_, hy⟩, hyQ⟩
          rintro rfl
          exact hrQ hyQ
      have hlt : (K.erase r).card < Nat.find hex := by
        rw [← hc]
        exact Finset.card_erase_lt_of_mem hrK
      exact Nat.find_min hex hlt ⟨K.erase r, hr, hsame, rfl⟩

theorem proj_atom {Q : Finset α} {q : α} (hqQ : q ∈ Q) (T : Finset α) :
    Atom (proj F Q) q T ↔
      AtomTrace F Q q T ∧ ∀ T', AtomTrace F Q q T' → T' ⊆ T → T' = T := by
  constructor
  · rintro ⟨⟨K, hK, rfl⟩, hqT, hmin⟩
    have hqK : q ∈ K := (Finset.mem_inter.mp hqT).1
    obtain ⟨A, hA, hAK⟩ := exists_atom hK hqK
    have hsub : A ∩ Q ⊆ K ∩ Q := Finset.inter_subset_inter_right hAK
    have heq : A ∩ Q = K ∩ Q :=
      hmin _ ⟨A, hA.1, rfl⟩ (Finset.mem_inter.mpr ⟨hA.2.1, hqQ⟩) hsub
    refine ⟨⟨A, hA, heq.symm⟩, ?_⟩
    rintro _ ⟨A', hA', rfl⟩ hle
    exact hmin _ ⟨A', hA'.1, rfl⟩ (Finset.mem_inter.mpr ⟨hA'.2.1, hqQ⟩) hle
  · rintro ⟨⟨A, hA, rfl⟩, hmin⟩
    refine ⟨⟨A, hA.1, rfl⟩, Finset.mem_inter.mpr ⟨hA.2.1, hqQ⟩, ?_⟩
    rintro B ⟨K, hK, rfl⟩ hqB hBT
    have hqK : q ∈ K := (Finset.mem_inter.mp hqB).1
    obtain ⟨A', hA', hA'K⟩ := exists_atom hK hqK
    have hle : A' ∩ Q ⊆ A ∩ Q := (Finset.inter_subset_inter_right hA'K).trans hBT
    have heq := hmin _ ⟨A', hA', rfl⟩ hle
    exact Finset.Subset.antisymm hBT (heq ▸ Finset.inter_subset_inter_right hA'K)

end Konekode
