# Visual/UI gate for a deterministic C kiosk -- Research Index

**Purpose:** evidence base for a deterministic, generic visual/UI gate (scene graph -> LVGL -> software framebuffer; Rust app as oracle; zero human review). See _SYNTHESIS.md.
**Sources summarised here:** 99 (82 with full-text PDF, 17 practitioner/standard/abstract-only) + ~90 reused summaries from earlier corpora (lui-*, event-reactor-*, rigor-levels-*), listed in the sub-topic reports.
**Date compiled:** 2026-11-10

## Reports
- _SYNTHESIS -- per-failure-mode techniques + cross-cutting recommendations
- _SUBTOPIC_A_E_layout_text -- layout/presentation failures, text rendering without OCR
- _SUBTOPIC_B_F_oracles_determinism -- oracles without humans, rendering determinism, golden governance
- _SUBTOPIC_C_exploration_temporal -- state exploration, coverage, temporal/HMI properties
- _SUBTOPIC_D_interaction -- hit-testing, target size, dead elements, feedback latency, dwell time

## All per-source summaries (Articles/)

| Source | Title | Full-text PDF |
|---|---|---|
| Alameer 2019 - i18n repair layout constraints | Efficiently Repairing Internationalization Presentation Failures By Solving Layout Constraints (CBRepair) | Yes |
| Alegroth 2015 - Visual GUI testing in practice | Visual GUI Testing in Practice: Challenges, Problems and Limitations | Yes |
| Alegroth 2018 - CI and visual GUI testing | Continuous Integration and Visual GUI Testing: Benefits and Drawbacks in Industrial Practice | Yes |
| Alshayban 2020 - Android accessibility issues | Accessibility Issues in Android Apps: State of Affairs, Sentiments, and Ways Forward | Yes |
| Alshayban 2022 - AccessiText | AccessiText: Automated Detection of Text Accessibility Issues in Android Apps | Yes |
| Amalfitano 2012 - Android GUI ripping | Using GUI Ripping for Automated Testing of Android Applications | Yes |
| Amalfitano 2015 - MobiGUITAR | MobiGUITAR: Automated Model-Based Testing of Mobile Apps | Yes |
| Android 2026 - Paparazzi and Roborazzi | Paparazzi (Cash App) and Roborazzi (takahirom) -- JVM-side Android screenshot testing | No |
| Ansys 2026 - SCADE Display qualified HMI | Ansys SCADE Display / SCADE Test / SCADE Solutions for ARINC 661 -- product claims | No |
| Apple Google 2025 - Platform target size and transient message guidance | Platform guidance on control size/spacing and transient-message durations: Apple Human Interface Guidelines (A | No |
| Badros 2001 - Cassowary | The Cassowary Linear Arithmetic Constraint Solving Algorithm | Yes |
| Baek 2016 - Multi-level GUI comparison criteria | Automated Model-Based Android GUI Testing using Multi-level GUI Comparison Criteria | Yes |
| Banerjee 2013 - GUI testing systematic mapping | Graphical User Interface (GUI) Testing: Systematic Mapping and Repository | Yes |
| Basin 2015 - Monitoring MFOTL | Monitoring Metric First-Order Temporal Properties | Yes |
| Bauer 2011 - Runtime verification LTL TLTL | Runtime Verification for LTL and TLTL | Yes |
| Belli 2001 - Event sequence graphs faulty sequences | Finite-State Testing and Analysis of Graphical User Interfaces | Yes |
| Bielik 2018 - InferUI relational layout synthesis | Robust Relational Layout Synthesis from Examples for Android (InferUI) | Yes |
| Bolton 2013 - Formal verification of HAI review | Using Formal Verification to Evaluate Human-Automation Interaction: A Review | Yes |
| Brysbaert 2019 - Reading rate meta-analysis | How many words do we read per minute? A review and meta-analysis of reading rate | Yes |
| Burkle 2024 - Instrument cluster rendering error detection | Learning-Based Error Detection System for Advanced Vehicle Instrument Cluster Rendering | Yes |
| Choi 2017 - SwiftHand minimal restart thesis | Guided GUI Testing of Android Apps with Minimal Restart and Approximate Learning (PhD thesis; contains SwiftHa | Yes |
| Chromium 2026 - Virtual time | Chromium virtual time: `--virtual-time-budget` and DevTools `Emulation.setVirtualTimePolicy` | No |
| Colle 2004 - Kiosk keypad key size spacing | Standing at a kiosk: Effects of key size and spacing on touch screen numeric keypad performance and user prefe | Yes |
| Dabrowski 2011 - 40 years response time | 40 years of searching for the best computer system response time | Yes |
| Degani 2002 - Formal verification of human-automation interaction | Formal Verification of Human-Automation Interaction | Yes |
| Dobres 2018 - Crowding text size glance legibility | The effects of visual crowding, text size, and positional uncertainty on text legibility at a glance | Yes |
| Dong 2020 - Time-travel testing Android | Time-travel Testing of Android Apps | Yes |
| Duan 2011 - MBT automotive HMI menus | Model-Based Testing for the Menu Behavior of Automotive Infotainment System HMIs | Yes |
| Dwyer 1999 - Property specification patterns | Patterns in Property Specifications for Finite-State Verification | Yes |
| Fazzini 2017 - DiffDroid cross-platform | Automated Cross-Platform Inconsistency Detection for Mobile Apps | Yes |
| Flutter 2026 - Golden tests and pump | Flutter `matchesGoldenFile`, `--update-goldens`, Ahem test font, `WidgetTester.pump/pumpAndSettle` | No |
| Fosco 2020 - Visual importance across design types | Predicting Visual Importance Across Graphic Design Types (UMSI) | Yes |
| Fratantonio 2017 - Cloak and Dagger | Cloak and Dagger: From Two Permissions to Complete Control of the UI Feedback Loop | Yes |
| FreeType-LVGL - Font metrics clipping check | (a) FreeType Glyph Conventions, "Glyph metrics" (https://freetype.org/freetype2/docs/glyphs/glyphs-3.html); (b | No |
| Fujita 2023 - Snapshot testing empirical | An Empirical Study on the Use of Snapshot Testing | Yes |
| Google 2025 - ATF touch target and duplicate clickable bounds | Accessibility Test Framework for Android (ATF): TouchTargetSizeCheck and DuplicateClickableBoundsCheck | No |
| Halle 2015 - Testing web apps through layout constraints | Testing Web Applications Through Layout Constraints (Cornipickle) | Yes |
| Harrison 2019 - UI verification templates | Verification Templates for the Analysis of User Interface Software Design | Yes |
| Henze 2011 - 100000000 taps | 100,000,000 Taps: Analysis and Improvement of Touch Performance in the Large | Yes |
| Ho 2014 - Online MTL monitoring | Online Monitoring of Metric Temporal Logic | Yes |
| Hoggan 2008 - Tactile feedback touchscreens | Investigating the effectiveness of tactile feedback for mobile touchscreens | Yes |
| Holz 2010 - Generalized perceived input point | The Generalized Perceived Input Point Model and How to Double Touch Accuracy by Extracting Fingerprints | Yes |
| Hu 2025 - Minuku display issues | Minuku: Detecting Diverse Display Issues in Mobile Apps with Small-scale Dataset | Yes |
| Iqbal 2019 - MBT cockpit display systems | A Model-based Testing Approach for Cockpit Display Systems of Avionics | Yes |
| Jin 2007 - Older adults button size spacing | Touch Screen User Interfaces for Older Adults: Button Size and Spacing | Yes |
| Kaaresoja 2016 - Latency guidelines virtual button feedback | Latency guidelines for touchscreen virtual button feedback (PhD thesis) | Yes |
| Kaynak 2025 - LLMShot snapshot maintenance | LLMShot: Reducing Snapshot Testing Maintenance via LLMs | Yes |
| Kong 2019 - Android automated testing SLR | Automated Testing of Android Apps: A Systematic Literature Review | Yes |
| Konrad 2005 - Real-time specification patterns | Real-time Specification Patterns | Yes |
| LVGL 2026 - Test module screenshot compare | LVGL test module: `lv_test_screenshot_compare`, `lv_test_wait`, `lv_test_fast_forward`, test display/indev | No |
| Legge 2011 - Print size reading | Does print size matter for reading? A review of findings from vision science and typography | Yes |
| Leitao 2012 - Older adults target spacing smartphone | Target and Spacing Sizes for Smartphone User Interfaces for Older Adults: Design Patterns Based on an Evaluati | Yes |
| Liu 2020 - OwlEyes | Owl Eyes: Spotting UI Display Issues via Visual Understanding | Yes |
| Liu 2022 - Nighthawk | Nighthawk: Fully Automated Localizing UI Display Issues via Visual Understanding | Yes |
| Mahajan 2014 - Image comparison presentation failures NIER | Finding HTML Presentation Failures Using Image Comparison Techniques | Yes |
| Mahajan 2015 - WebSee | Detection and Localization of HTML Presentation Failures Using Computer Vision-Based Techniques (WebSee) | Yes |
| Mahajan 2016 - Visual symptoms debugging | Using Visual Symptoms for Debugging Presentation Failures in Web Applications (FieryEye) | Yes |
| Mahajan 2018 - MFix mobile friendly | Automated Repair of Mobile Friendly Problems in Web Pages (MFix) | Yes |
| McKeeman 1998 - Differential testing | Differential Testing for Software | Yes |
| Memon 2000 - Automated test oracles for GUIs | Automated Test Oracles for GUIs | Yes |
| Memon 2001 - GUI coverage criteria | Coverage Criteria for GUI Testing | Yes |
| Memon 2003 - GUI ripping | GUI Ripping: Reverse Engineering of Graphical User Interfaces for Testing | Yes |
| Memon 2003 - What test oracle for GUI testing | What Test Oracle Should I Use for Effective GUI Testing? | Yes |
| Mesbah 2012 - Crawljax UI state crawling | Crawling AJAX-Based Web Applications through Dynamic Analysis of User Interface State Changes | Yes |
| Mesbah 2012 - Invariant-based web testing | Invariant-Based Automatic Testing of Modern Web Applications | Yes |
| Microsoft 2025 - Clarity dead and rage clicks | Microsoft Clarity "Semantic metrics": dead clicks, rage clicks, quick backs, click errors | No |
| Miller 1968 - Response time conversational | Response time in man-computer conversational transactions | No |
| Nguyen 2014 - GUITAR | GUITAR: An Innovative Tool for Automated Testing of GUI-Driven Software | Yes |
| Nielsen 1993 - Response time limits | Response Times: The 3 Important Limits (excerpt of Usability Engineering, 1993; 2014 update); plus heuristic # | No |
| Norman 1981 - Action slips | Categorization of Action Slips | Yes |
| OConnor 2022 - Quickstrom LTL acceptance testing | Quickstrom: Property-based Acceptance Testing with LTL Specifications | Yes |
| ODonovan 2014 - Learning layouts graphic designs | Learning Layouts for Single-Page Graphic Designs | Yes |
| Oladimeji 2011 - Number entry error detection | Number Entry Interfaces and Their Effects on Error Detection | Yes |
| Parhi 2006 - Thumb target size | Target size study for one-handed thumb use on small touchscreen devices | Yes |
| Park 2010 - Touch key size location thumb | Touch key design for one-handed thumb interaction with a mobile phone: Effects of touch key size and touch key | Yes |
| Parry 2021 - Flaky tests survey | A Survey of Flaky Tests | Yes |
| Playwright 2026 - toHaveScreenshot | Playwright `expect(page).toHaveScreenshot()` options | No |
| Practitioner 2022 - Pseudolocalisation and text expansion | (a) Microsoft Learn, "Pseudolocalization" (Globalization methodology, ms.date 2022-06-07); (b) Android Develop | No |
| Rushby 2002 - Model checking mode confusion | Using Model Checking to Help Discover Mode Confusions and Other Automation Surprises | Yes |
| Ryou 2018 - Visibility faults HTML5 | Automatic Detection of Visibility Faults by Layout Changes in HTML5 Web Pages (VFDetector) | Yes |
| Sellen 1992 - Mode errors sensory feedback | The Prevention of Mode Errors Through Sensory Feedback | Yes |
| Sesto 2012 - Button size spacing disabilities | Effect of Touch Screen Button Size and Spacing on Touch Characteristics of Users With and Without Disabilities | Yes |
| Strecker 2008 - GUI test suites faults detection | Relationships Between Test Suites, Faults, and Fault Detection in GUI Testing | Yes |
| Su 2021 - Genie functional fuzzing | Fully Automated Functional Fuzzing of Android Apps for Detecting Non-crashing Logic Bugs | Yes |
| Su 2021 - Themis Android GUI testing benchmark | Benchmarking Automated GUI Testing for Android against Real-World Bugs | Yes |
| Su 2022 - dVermin scaling issues | The Metamorphosis: Automatic Detection of Scaling Issues for Mobile Apps | Yes |
| Sun 2021 - SetDroid system settings | Understanding and Finding System Setting-Related Defects in Android Apps | Yes |
| Thimbleby 2010 - Reducing number entry errors | Reducing Number Entry Errors: Solving a Widespread, Serious Problem | Yes |
| Thorve 2018 - Android flaky tests | An Empirical Study of Flaky Tests in Android Apps | Yes |
| W3C 2023 - WCAG 2.2 target size | WCAG 2.2 Success Criteria 2.5.8 Target Size (Minimum) (AA) and 2.5.5 Target Size (Enhanced) (AAA), with "Under | No |
| W3C 2023 - WCAG 2.2 timing adjustable | WCAG 2.2 Success Criterion 2.2.1 Timing Adjustable (Level A), Understanding document | No |
| W3C Playwright 2025 - Hit-test actionability checks | WebDriver "Element Click" interactability algorithm (W3C) and Playwright actionability checks | No |
| W3C-WAI 2023 - WCAG 2.2 text resize reflow spacing | Web Content Accessibility Guidelines (WCAG) 2.2 -- SC 1.4.3, 1.4.4, 1.4.6, 1.4.10, 1.4.11, 1.4.12, 2.5.8 and g | No |
| Wang 2019 - Textout | Textout: Detecting Text-Layout Bugs in Mobile Apps via Visualization-Oriented Learning | Yes |
| Wang 2022 - ODIN deep-state differential | Detecting Non-crashing Functional Bugs in Android Apps via Deep-State Differential Analysis | Yes |
| Xie 2007 - Comparing GUI test oracles | Designing and Comparing Automated Test Oracles for GUI-Based Software Applications | Yes |
| Xiong 2024 - Kea property-based Android | General and Practical Property-based Testing for Android Apps | Yes |
| Yandrapally 2020 - Near-duplicate state detection | Near-Duplicate Detection in Web App Model Inference | Yes |
| Zaeem 2014 - QUANTUM interaction oracles | Automated Generation of Oracles for Testing User-Interaction Features of Mobile Apps | Yes |

*99 sources, 82 PDFs (~136 MB), compiled 2026-11-10. Copyrighted full texts are local only.*
