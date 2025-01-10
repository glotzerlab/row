// Copyright (c) 2024-2025 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use clap_complete::CompletionCandidate;
use indicatif::{MultiProgress, ProgressDrawTarget};

use row::workflow::Workflow;
use row::MultiProgressContainer;
use row::{cluster, workspace};

/// List the actions in the current workflow.
pub fn get_action_candidates() -> Vec<CompletionCandidate> {
    let Ok(workflow) = Workflow::open() else {
        return Vec::new();
    };

    workflow
        .action
        .into_iter()
        .map(|a| CompletionCandidate::new(a.name()))
        .collect()
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
        .collect()
}

/// List the directories in the project's workspace
pub fn get_directory_candidates() -> Vec<CompletionCandidate> {
    let multi_progress = MultiProgress::with_draw_target(ProgressDrawTarget::hidden());
    let mut multi_progress_container = MultiProgressContainer::new(multi_progress.clone());

    let Ok(workflow) = Workflow::open() else {
        return Vec::new();
    };

    let Ok(directories) = workspace::list_directories(&workflow, &mut multi_progress_container)
    else {
        return Vec::new();
    };

    directories
        .into_iter()
        .map(CompletionCandidate::new)
        .collect()
}
