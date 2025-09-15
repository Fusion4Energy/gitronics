from gitronics.project_checker import NewProjectChecker
from gitronics.project_manager import ProjectManager

project_manager = ProjectManager(project_root="example_project")
project_checker = NewProjectChecker(project_manager=project_manager)
project_checker.check_project()
