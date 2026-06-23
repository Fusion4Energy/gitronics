# Assessments

Gitronics can also be used to manage the specific changes required to the model for a specific assessment.
The recommended practice is to create a new Git branch for the assessment.
Then, create a new directory that will contain all the assessment-specific files (e.g. new filler models, tallies, configuration files, etc.).
This directory could also contain post-processing scripts, organization files and others.

Remember that the `project_roots` field in the configuration file can contain multiple directories, so the assessment-specific directory can be added to the list of roots to be searched for files.

```
gitronics_project/
├── configurations/
│   └── baseline.yaml             ← The reference configuration file
|      
├── reference_model/              ← Directory containing the reference files
│   └── ...
|
└── assessment_specific/
    ├── configurations/
    │   └── modified.yaml         ← This configuration overrides baseline.yaml applying changes
    │
    ├── changes/                  ← Directory containing new files
    │   ├── specific_tally.tally  ← New tally only for this assessment
    │   └── new_diagnostic.yaml   ← New filler only for this assessment
    |
    ├── output/
    │   └── assembled.mcnp        ← Assembled model for this assessment
    │   
    └── postprocessing/           ← Scripts to process the results of this assessment
        └── ... 
```

After the assessment is complete, the branch can be stored as a file via `git bundle`, and the assessment-specific files can be deleted from the main branch to keep it clean.
