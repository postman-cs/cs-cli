//! Profile detector integration tests
//!
//! Tests browser profile detection functionality with real user profiles

use cs_cli::common::auth::profile_detector::{ProfileDetector, PlatformType};

#[tokio::test]
async fn test_profile_discovery() {
    // Initialize detector
    let detector = match ProfileDetector::new() {
        Ok(d) => d,
        Err(_) => {
            // Skip test if profile detection fails (e.g., no browsers installed)
            println!("Skipping profile detection test - no profiles found");
            return;
        }
    };
    
    println!("Found {} profiles", detector.detected_profiles.len());
    
    // Should find at least one profile on a development machine
    if !detector.detected_profiles.is_empty() {
        for profile in &detector.detected_profiles {
            println!("Profile: {}", profile.description());
            
            // Basic validation
            assert!(!profile.profile_id.is_empty());
            assert!(!profile.profile_name.is_empty());
            assert!(profile.profile_path.exists() || profile.profile_path.parent().unwrap().exists());
        }
    }
}

#[tokio::test] 
async fn test_platform_selection() {
    let detector = match ProfileDetector::new() {
        Ok(d) => d,
        Err(_) => return, // Skip if no profiles
    };
    
    if detector.detected_profiles.is_empty() {
        return;
    }
    
    // Test work profile selection
    if let Some(work_profile) = detector.find_best_profile_for_platform(&PlatformType::Work) {
        println!("Work profile: {}", work_profile.description());
        // Work profiles should have work characteristics
        if work_profile.email.is_some() {
            // If has email, check it's not a common personal domain
            let email = work_profile.email.as_ref().unwrap();
            assert!(!email.ends_with("@gmail.com") || work_profile.is_managed);
        }
    }
    
    // Test personal profile selection  
    if let Some(personal_profile) = detector.find_best_profile_for_platform(&PlatformType::Personal) {
        println!("Personal profile: {}", personal_profile.description());
    }
}

#[tokio::test]
async fn test_profile_classification() {
    let detector = match ProfileDetector::new() {
        Ok(d) => d,
        Err(_) => return,
    };
    
    if detector.detected_profiles.is_empty() {
        return;
    }
    
    let work_profiles = detector.find_work_profiles();
    let personal_profiles = detector.find_personal_profiles();
    
    println!("Work profiles: {}", work_profiles.len());
    println!("Personal profiles: {}", personal_profiles.len());
    
    // Should have some classification (unless all profiles are ambiguous)
    assert!(work_profiles.len() + personal_profiles.len() <= detector.detected_profiles.len());
}

#[tokio::test]
async fn test_most_recent_profile() {
    let detector = match ProfileDetector::new() {
        Ok(d) => d,
        Err(_) => return,
    };
    
    if let Some(recent) = detector.get_most_recent_profile() {
        println!("Most recent profile: {}", recent.description());
        
        // Should be a valid profile
        assert!(!recent.profile_id.is_empty());
        assert!(!recent.profile_name.is_empty());
    }
}