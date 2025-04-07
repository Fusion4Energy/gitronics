.. Gitronics documentation master file, created by
   sphinx-quickstart on Mon Apr  7 19:01:49 2025.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

Gitronics documentation
=======================

Welcome to the Gironics documentation. Gitronics is a Python package that allows the
modularization of MCNP models. It is designed to help users create and manage complex 
MCNP models by breaking them down into smaller, single-responsibility files.

Installation
============

To install Gitronics you can clone the repository and then install it in a Python 
environment using pip.

How to use
==========

Inside the Gitronics package there is a folder called *example_project*. This folder
can serve as a template for your own projects as it contains a working example of a 
MCNP model using Gitronics.

Your project may have a different directory structure, it does not matter where each
file is stored or how they are grouped, as long as the paths in the `build_model.py`
script and the `project_summary.csv` are correctly set.

Philosophy
----------

The philosophy behind Gitronics is to consider an MCNP reference model as a project 
that contains several files representing the different data that would define a MCNP
model like the geometry of a system, material definitions, particle sources, tallies, 
etc.
Then, according to a configuration file, Gitronics will automatically compile all the 
different files into a single MCNP input file which will be referred as 
`assembled.mcnp`.

In Gitronics, there are 6 types of files:

- Main input file: 
   Can also be called envelope structure. This file contains the 
   envelope cells that will be filled by universe models. In the case of a simple model
   without universes, this file will contain the geometry of the system. It has the 
   suffix `.mcnp`.
- Universe model: 
   This file contains the geometry of a MCNP universe which often 
   represents a specific system. The project can contain many universe models, and not 
   all of them may be needed for a specific configuration. They have the suffix `.mcnp`.
- Data cards: 
   These files contain the data cards that may be used in the MCNP input 
   file. They can be used to define materials, sources, tallies, etc. They have the 
   suffix `.mat`, `.source`, `.tally` or `transform`. Only a single `.source` file is 
   allowed to be selected in a configuration.
- Configuration files: 
   Each of these files represent a specific configuration of the
   project. They are used to select which files will be included in the final MCNP
   input file. They are in YAML format.
- Project summary: 
   This file contains information about each of the other files of 
   the project. It is needed to build the `assembled.mcnp`. 
- Building script: 
   This script is used to build the `assembled.mcnp` file. It is a 
   Python file that makes use of the Gitronics package. It is called `build_model.py`.

Main input file
---------------

This file contains the envelope cells that will be filled with universes models. To
define a cell as an envelope, it must include a comment of the form `$ FILL = <name>`, 
where `<name>` is the name of the envelope (e.g. `$ FILL = Equatorial Port 04`).

Universe model
--------------

The file contains the geometry of a MCNP universe. There should be no title card at
the beginning of the file. The cells should contain the `U = <universe id>` card that
defines them as a universe.

Data cards
----------

The file represents a data card or set of data cards that may be included in the MCNP
model. Anything written after a blank line will not be considered.

Configuration files
-------------------

A configuration file is a YAML file used to select which files will be included in the
`assembled.mcnp` file. The fields in the configuration are filled with names of other
files. These names are not the file name of the file but a given name that is defined in
the project summary file.
The configuration may contain any of the following fields:

- overrides: 
   This field can be filled with the name of another configuration file that 
   will serve as default. Any other field filled will override the default value.
- envelope_structure: 
   The name of the main input file.
- envelopes: 
   Filled with a dict of the form:
   ``{<envelope name>: {filler: <universe name>, transform: <transform>}}``. The envelope name
   must be the same as the one used in the main input file. The transform is optional, 
   and is a string that will be added after the ``FILL = 123`` card and therefore can be used
   to define a transformation like in ``FILL = 123  10 10 10``.
- source: 
   The name of the source file. Only one source file is allowed.
- tallies: 
   A list of the names of the tally files that will be included in the MCNP 
   model.
- materials: 
   A list of the names of the material files that will be included in the 
   MCNP model.
- transforms: 
   A list of the names of the transform files that will be included in the
   MCNP model.

Project summary
---------------

It is a CSV file that can contain any information the user wants to add about the
files. It is mandatory to include as the first two columns: *Name* and *Relative path*.
Each row represents a file in the project. The *Name* column is the name that will be 
used in the configuration file. The *Relative path* column is the path to the file 
relative to the project root.

Building script
---------------

The user should update the paths inside the script before running it. In the script, it
is specified which configuration file will be used. Run the script to generated the 
`assembled.mcnp` file.

.. toctree::
   :maxdepth: 2
   :caption: Contents:

