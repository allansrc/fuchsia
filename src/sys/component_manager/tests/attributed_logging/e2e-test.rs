// Copyright 2020 The Fuchsia Authors. All rights reserved.
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use {
    component_events::{
        events::{self, Event},
        matcher::{EventMatcher, ExitStatusMatcher},
        sequence::{self, EventSequence},
    },
    fuchsia_async as fasync,
    test_utils_lib::opaque_test::OpaqueTestBuilder,
};

#[fasync::run_singlethreaded(test)]
/// Verifies that when a component has a LogSink in its namespace that the
/// component manager tries to connect to this and that if that component
/// tries to use a capability it isn't offered we see the expect log content
/// which is generated by component manager, but should be attributed to the
/// component that tried to use the capability.
///
/// Part of the test runs inside the `reader` component inside the test
/// topology. If `reader` sees the log it expects from the Archvist component
/// it exits cleanly, otherwise it crashes.
async fn verify_routing_failure_messages() {
    let test_env = OpaqueTestBuilder::new(
        "fuchsia-pkg://fuchsia.com/attributed-logging-test#meta/e2e-root.cm",
    )
    .component_manager_url(
        "fuchsia-pkg://fuchsia.com/attributed-logging-test#meta/component-manager.cmx",
    )
    .config("/pkg/data/cm_config")
    .build()
    .await
    .expect("failed to construct OpaqueTest");

    let mut event_source = test_env
        .connect_to_event_source()
        .await
        .expect("could not connect to event source for opaque test");

    let expected = EventSequence::new()
        .all_of(
            vec![
                EventMatcher::ok().r#type(events::Stopped::TYPE).moniker_regex("/routing-tests/child"),
                EventMatcher::ok().r#type(events::Stopped::TYPE).moniker_regex(
                    "/routing-tests/offers-to-children-unavailable/child-for-offer-from-parent",
                ),
                EventMatcher::ok().r#type(events::Stopped::TYPE).moniker_regex(
                    "/routing-tests/offers-to-children-unavailable/child-for-offer-from-sibling",
                ),
                EventMatcher::ok().r#type(events::Stopped::TYPE).moniker_regex(
                    "/routing-tests/offers-to-children-unavailable/child-open-unrequested",
                ),
                EventMatcher::ok()
                    .r#type(events::Stopped::TYPE)
                    .moniker_regex("/reader")
                    .stop(Some(ExitStatusMatcher::Clean)),
                EventMatcher::ok()
                    .r#type(events::CapabilityRouted::TYPE)
                    .capability_name("fuchsia.diagnostics.ArchiveAccessor")
                    .moniker_regex("/reader"),
                EventMatcher::ok()
                    .r#type(events::CapabilityRouted::TYPE)
                    .capability_name("fuchsia.logger.LogSink")
                    .moniker_regex("/routing-tests/child"),
                EventMatcher::ok()
                    .r#type(events::CapabilityRouted::TYPE)
                    .capability_name("fuchsia.logger.LogSink")
                    .moniker_regex(
                        "/routing-tests/offers-to-children-unavailable/child-for-offer-from-parent",
                    ),
                EventMatcher::ok()
                    .r#type(events::CapabilityRouted::TYPE)
                    .capability_name("fuchsia.logger.LogSink")
                    .moniker_regex(
                        "/routing-tests/offers-to-children-unavailable/child-for-offer-from-sibling",
                    ),
                EventMatcher::ok()
                    .r#type(events::CapabilityRouted::TYPE)
                    .capability_name("fuchsia.logger.LogSink")
                    .moniker_regex(
                        "/routing-tests/offers-to-children-unavailable/child-open-unrequested",
                    ),
                EventMatcher::ok()
                    .r#type(events::CapabilityRouted::TYPE)
                    .capability_name("fuchsia.logger.LogSink")
                    .moniker_regex("/archivist"),
                EventMatcher::ok()
                    .r#type(events::CapabilityRouted::TYPE)
                    .capability_name("fuchsia.sys2.EventSource")
                    .moniker_regex("/archivist"),
                EventMatcher::ok()
                    .r#type(events::CapabilityRouted::TYPE)
                    .capability_name("fuchsia.logger.LogSink")
                    .moniker_regex("/archivist"),
            ],
            sequence::Ordering::Unordered,
        )
        .subscribe_and_expect(&mut event_source)
        .await
        .unwrap();

    event_source.start_component_tree().await;
    expected.await.unwrap();
}