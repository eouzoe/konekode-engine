module

public import KonekodeProofs.Space

@[expose] public section

namespace Konekode

variable {α : Type*} [DecidableEq α] {F : Set (Finset α)}

def Child (F : Set (Finset α)) (Q T M : Finset α) : Set (Finset α) :=
  {X | ∃ L ∈ F, L ∩ Q = T ∧ X = L \ M}

def Plus (F : Set (Finset α)) (Q T M : Finset α) : Set (Finset α) :=
  {X | X ∈ Child F Q T M ∨ X = ∅}

def Bottom (F : Set (Finset α)) (Q T : Finset α) (L : Finset α) : Prop :=
  L ∈ F ∧ L ∩ Q = T ∧ ∀ L' ∈ F, L' ∩ Q = T → L' ⊆ L → L' = L

def Lower (F : Set (Finset α)) (Q T M : Finset α) : Prop :=
  ∀ L ∈ F, L ∩ Q = T → M ⊆ L

theorem subset_of_sdiff_subset {L L' M : Finset α} (hM : M ⊆ L') (h : L \ M ⊆ L' \ M) :
    L ⊆ L' := by
  intro y hy
  by_cases hyM : y ∈ M
  · exact hM hyM
  · exact (Finset.mem_sdiff.mp (h (Finset.mem_sdiff.mpr ⟨hy, hyM⟩))).1

theorem child_union (h : LearningSpace F) {Q T M X Y : Finset α}
    (hX : X ∈ Child F Q T M) (hY : Y ∈ Child F Q T M) : X ∪ Y ∈ Child F Q T M := by
  obtain ⟨L, hL, hLT, rfl⟩ := hX
  obtain ⟨L', hL', hL'T, rfl⟩ := hY
  refine ⟨L ∪ L', h.union hL hL', ?_, ?_⟩
  · rw [Finset.union_inter_distrib_right, hLT, hL'T, Finset.union_self]
  · ext y
    simp only [Finset.mem_sdiff, Finset.mem_union]
    tauto

theorem child_augment (h : LearningSpace F) {Q T M X Y : Finset α} (hM : Lower F Q T M)
    (hX : X ∈ Child F Q T M) (hY : Y ∈ Child F Q T M) (hXY : X ⊆ Y) (hne : X ≠ Y) :
    ∃ y ∈ Y, y ∉ X ∧ insert y X ∈ Child F Q T M := by
  obtain ⟨L, hL, hLT, rfl⟩ := hX
  obtain ⟨L', hL', hL'T, rfl⟩ := hY
  have hsub : L ⊆ L' := subset_of_sdiff_subset (hM L' hL' hL'T) hXY
  have hnot : ¬ L' ⊆ L := fun hs => hne (by rw [Finset.Subset.antisymm hsub hs])
  obtain ⟨x, hxL', hxL, hins⟩ := augment h hL hL' hnot
  have hxQ : x ∉ Q := by
    intro hxQ
    have hxT : x ∈ T := by
      rw [← hL'T]
      exact Finset.mem_inter.mpr ⟨hxL', hxQ⟩
    rw [← hLT] at hxT
    exact hxL (Finset.mem_inter.mp hxT).1
  have hxM : x ∉ M := fun hxM => hxL (hM L hL hLT hxM)
  refine ⟨x, Finset.mem_sdiff.mpr ⟨hxL', hxM⟩, fun hx => hxL (Finset.mem_sdiff.mp hx).1,
    insert x L, hins, ?_, ?_⟩
  · rw [← hLT]
    ext y
    simp only [Finset.mem_inter, Finset.mem_insert]
    constructor
    · rintro ⟨rfl | hy, hyQ⟩
      · exact absurd hyQ hxQ
      · exact ⟨hy, hyQ⟩
    · rintro ⟨hy, hyQ⟩
      exact ⟨Or.inr hy, hyQ⟩
  · ext y
    simp only [Finset.mem_insert, Finset.mem_sdiff]
    constructor
    · rintro (rfl | ⟨hy, hyM⟩)
      · exact ⟨Or.inl rfl, hxM⟩
      · exact ⟨Or.inr hy, hyM⟩
    · rintro ⟨rfl | hy, hyM⟩
      · exact Or.inl rfl
      · exact Or.inr ⟨hy, hyM⟩

theorem access_of_augment (G : Set (Finset α))
    (haug : ∀ X Y, X ∈ G → Y ∈ G → X ⊆ Y → X ≠ Y → ∃ y ∈ Y, y ∉ X ∧ insert y X ∈ G) :
    ∀ (n : ℕ) (X Y : Finset α), X ∈ G → Y ∈ G → X ⊆ Y → (Y \ X).card = n + 1 →
      ∃ y ∈ Y, Y.erase y ∈ G := by
  intro n
  induction n with
  | zero =>
    intro X Y hX hY hXY hc
    obtain ⟨a, ha⟩ := Finset.card_eq_one.mp hc
    have hne : X ≠ Y := by
      rintro rfl
      simp at hc
    obtain ⟨y, hyY, hyX, _⟩ := haug X Y hX hY hXY hne
    refine ⟨y, hyY, ?_⟩
    have hyA : y = a := by
      have : y ∈ Y \ X := Finset.mem_sdiff.mpr ⟨hyY, hyX⟩
      rw [ha] at this
      exact Finset.mem_singleton.mp this
    have heq : Y.erase y = X := by
      ext z
      simp only [Finset.mem_erase]
      constructor
      · rintro ⟨hzy, hzY⟩
        by_contra hzX
        have : z ∈ Y \ X := Finset.mem_sdiff.mpr ⟨hzY, hzX⟩
        rw [ha] at this
        exact hzy ((Finset.mem_singleton.mp this).trans hyA.symm)
      · intro hzX
        exact ⟨fun hzy => hyX (hzy ▸ hzX), hXY hzX⟩
    rw [heq]
    exact hX
  | succ n ih =>
    intro X Y hX hY hXY hc
    have hne : X ≠ Y := by
      rintro rfl
      simp at hc
    obtain ⟨y, hyY, hyX, hins⟩ := haug X Y hX hY hXY hne
    have hsub : insert y X ⊆ Y := Finset.insert_subset hyY hXY
    have hc' : (Y \ insert y X).card = n + 1 := by
      have hset : Y \ insert y X = (Y \ X).erase y := by
        ext z
        simp only [Finset.mem_sdiff, Finset.mem_insert, Finset.mem_erase]
        tauto
      rw [hset, Finset.card_erase_of_mem (Finset.mem_sdiff.mpr ⟨hyY, hyX⟩), hc]
      omega
    exact ih (insert y X) Y hins hY hsub hc'

theorem exists_bottom {Q T L : Finset α} (hL : L ∈ F) (hLT : L ∩ Q = T) :
    ∃ L₀, Bottom F Q T L₀ ∧ L₀ ⊆ L := by
  classical
  let S := L.powerset.filter (fun B => B ∈ F ∧ B ∩ Q = T)
  have hS : S.Nonempty :=
    ⟨L, Finset.mem_filter.mpr ⟨Finset.mem_powerset_self L, hL, hLT⟩⟩
  obtain ⟨B, hB, hmin⟩ := Finset.exists_min_image S Finset.card hS
  obtain ⟨hBL', hBF, hBT⟩ := Finset.mem_filter.mp hB
  have hBL := Finset.mem_powerset.mp hBL'
  refine ⟨B, ⟨hBF, hBT, fun B' hB' hB'T hB'B => ?_⟩, hBL⟩
  have hB'S : B' ∈ S :=
    Finset.mem_filter.mpr ⟨Finset.mem_powerset.mpr (hB'B.trans hBL), hB', hB'T⟩
  exact Finset.eq_of_subset_of_card_le hB'B (hmin B' hB'S)

theorem yielding_iff (h : LearningSpace F) {Q T M : Finset α} (hM : Lower F Q T M) :
    (∀ L, Bottom F Q T L → (L \ M).card ≤ 1) ↔ LearningSpace (Plus F Q T M) := by
  constructor
  · intro hyield
    have haug : ∀ X Y, X ∈ Plus F Q T M → Y ∈ Plus F Q T M → X ⊆ Y → X ≠ Y →
        ∃ y ∈ Y, y ∉ X ∧ insert y X ∈ Plus F Q T M := by
      rintro X Y (hX | rfl) (hY | rfl) hXY hne
      · obtain ⟨y, hy, hyX, hins⟩ := child_augment h hM hX hY hXY hne
        exact ⟨y, hy, hyX, Or.inl hins⟩
      · exact absurd (Finset.subset_empty.mp hXY) hne
      · obtain ⟨L, hL, hLT, rfl⟩ := hY
        obtain ⟨L₀, hL₀, hL₀L⟩ := exists_bottom hL hLT
        have hle := hyield L₀ hL₀
        by_cases hzero : (L₀ \ M).card = 0
        · have hempty : L₀ \ M = ∅ := Finset.card_eq_zero.mp hzero
          have hX : (∅ : Finset α) ∈ Child F Q T M := ⟨L₀, hL₀.1, hL₀.2.1, hempty.symm⟩
          obtain ⟨y, hy, hyX, hins⟩ :=
            child_augment h hM hX ⟨L, hL, hLT, rfl⟩ hXY hne
          exact ⟨y, hy, hyX, Or.inl hins⟩
        · obtain ⟨y, hy⟩ := Finset.card_eq_one.mp (show (L₀ \ M).card = 1 by omega)
          have hyL : y ∈ L₀ \ M := by
            rw [hy]
            exact Finset.mem_singleton_self y
          refine ⟨y, ?_, Finset.notMem_empty y, Or.inl ⟨L₀, hL₀.1, hL₀.2.1, ?_⟩⟩
          · obtain ⟨hy₁, hy₂⟩ := Finset.mem_sdiff.mp hyL
            exact Finset.mem_sdiff.mpr ⟨hL₀L hy₁, hy₂⟩
          · rw [hy]
            rfl
      · exact absurd rfl hne
    refine ⟨Or.inr rfl, ?_, ?_⟩
    · rintro X Y (hX | rfl) (hY | rfl)
      · exact Or.inl (child_union h hX hY)
      · rw [Finset.union_empty]
        exact Or.inl hX
      · rw [Finset.empty_union]
        exact Or.inl hY
      · rw [Finset.union_empty]
        exact Or.inr rfl
    · intro Y hY hne
      obtain ⟨n, hn⟩ : ∃ n, (Y \ ∅).card = n + 1 := by
        rw [Finset.sdiff_empty]
        exact ⟨Y.card - 1, by have := Finset.card_pos.mpr hne; omega⟩
      exact access_of_augment _ haug n ∅ Y (Or.inr rfl) hY (Finset.empty_subset Y) hn
  · intro hplus L hL
    by_contra hbig
    have hmem : L \ M ∈ Plus F Q T M := Or.inl ⟨L, hL.1, hL.2.1, rfl⟩
    have hne : (L \ M).Nonempty := by
      rw [← Finset.card_pos]
      omega
    obtain ⟨r, hr, hrest⟩ := hplus.access hmem hne
    rcases hrest with ⟨L', hL', hL'T, heq⟩ | hempty
    · have hsub : L' \ M ⊆ L \ M := heq ▸ Finset.erase_subset r (L \ M)
      have hL'L : L' ⊆ L := subset_of_sdiff_subset (hM L hL.1 hL.2.1) hsub
      have hsame := hL.2.2 L' hL' hL'T hL'L
      have hrnot : r ∉ (L \ M).erase r := by simp
      rw [heq, hsame] at hrnot
      exact hrnot hr
    · have hc := Finset.card_erase_of_mem hr
      rw [hempty, Finset.card_empty] at hc
      omega

end Konekode
