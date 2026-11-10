# Tasks: AVD Emulator Setup for mdreader/Ressoa

**Input**: Design documents from `/specs/011-avd-emulator-setup/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: No explicit tests requested in feature specification. Following infrastructure/setup workflow.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., [US1], [US2], [US3])

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Verify environment and prerequisites

- [x] T001 Verify Android SDK installation at `~/Android/Sdk` and required components (emulator, platform-tools, cmdline-tools)
- [x] T002 [P] Verify KVM availability and permissions for hardware acceleration
- [x] T003 [P] Verify existing AVD configuration for `mdreader_avd` exists and is accessible

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Check AVD configuration details (API level, ABI, device profile, GPU settings) against requirements
- [x] T005 Verify ADB is available and functional

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Verify and Start Existing AVD (Priority: P1) 🎯 MVP

**Goal**: Verify the pre-configured AVD is appropriate and start it with proper hardware acceleration

**Independent Test**: Can be tested by checking AVD configuration, starting emulator, and verifying ADB connectivity without building/installing the app

### Implementation for User Story 1

- [x] T006 [P] [US1] Review AVD config.ini and document current settings
- [x] T007 [US1] Start emulator with appropriate flags (`-gpu swiftshader_indirect -no-boot-anim -no-audio`) per quickstart
- [x] T008 [US1] Wait for emulator boot completion and verify ADB connectivity
- [x] T009 [US1] Document emulator startup process and any issues encountered

**Checkpoint**: At this point, User Story 1 should be fully functional - emulator running and connected via ADB

---

## Phase 4: User Story 2 - Build and Install App on Emulator (Priority: P1)

**Goal**: Build the mdreader Android artifact and install it on the emulator

**Independent Test**: Can be tested by building for appropriate ABI and installing via ADB

### Implementation for User Story 2

- [ ] T010 [P] [US2] Determine appropriate build target (x86_64 for emulator testing, aarch64 for gate 9)
- [ ] T011 [US2] Build Android artifact with Tauri for selected target
- [ ] T012 [US2] Install APK on running emulator via ADB
- [ ] T013 [US2] Launch app and verify it starts without immediate crashes
- [ ] T014 [US2] Document build/install process and ABI considerations

**Checkpoint**: At this point, User Stories 1 AND 2 should both work - emulator running with app installed

---

## Phase 5: User Story 3 - Execute Structural Validation on Emulator (Priority: P2)

**Goal**: Follow structural verification procedures from specs/008-ref-device-emulator

**Independent Test**: Can be tested by following quickstart procedures and producing VERIF-EMULATOR attestation

### Implementation for User Story 3

- [ ] T015 [P] [US3] Review specs/008-ref-device-emulator/quickstart.md procedures
- [ ] T016 [US3] Execute structural validation steps (notification, focus/ducking/interruption, resume by fingerprint, diag JSONL)
- [ ] T017 [US3] Document SC-009 measurement approach with emulator caveats if needed
- [ ] T018 [US3] Create VERIF-EMULATOR attestation following project template/requirements

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T019 [P] Update documentation with emulator setup procedures and findings
- [ ] T020 Document troubleshooting notes and common issues
- [ ] T021 Create cleanup/shutdown procedures for emulator
- [ ] T022 Validate quickstart.md instructions work end-to-end
- [ ] T023 Review and document any gaps identified in checklists

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - US1 can start after Foundational
  - US2 can start after Foundational (may benefit from US1 running)
  - US3 can start after US2 (needs app installed)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational - Needs emulator running (can use US1 result)
- **User Story 3 (P2)**: Depends on US2 (needs app installed on emulator)

### Within Each User Story

- Core implementation before integration/documentation
- Story complete before moving to next priority

### Parallel Opportunities

- T002, T003 can run in parallel in Setup phase
- T006 can run in parallel in US1
- T010 can run in parallel in US2
- T015 can run in parallel in US3
- T019 can run in parallel in Polish phase

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test emulator boots and ADB connects independently
5. This unblocks basic device/emulator access

### Incremental Delivery

1. Setup + Foundational → Environment verified
2. Add User Story 1 → Emulator running (MVP)
3. Add User Story 2 → App installed and running
4. Add User Story 3 → Structural validation complete
5. Each story adds value without breaking previous stories
