// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use clap_complete::CompletionCandidate;
use row::cluster;
use row::workflow::Workflow;

/// List the actions in the current workflow.
pub fn get_action_candidates() -> Vec<CompletionCandidate> {
    let Ok(workflow) = Workflow::open() else {
        return Vec::new();
    };

    workflow
        .action
        .into_iter()
        .map(|a| CompletionCandidate::new(a.name()))
        .collect::<Vec<_>>()
}

/// List the clusters in the user's configuration
pub fn get_cluster_candidates() -> Vec<CompletionCandidate> {
    let Ok(clusters) = cluster::Configuration::open() else {
        return Vec::new();
    };

    clusters
        .cluster
        .into_iter()
        .map(|a| CompletionCandidate::new(a.name))
        .collect::<Vec<_>>()
}
