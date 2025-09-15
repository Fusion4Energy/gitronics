from gitronics.project_checker import ProjectChecker
from gitronics.project_manager import ProjectManager

project_manager = ProjectManager(project_root="example_project")
project_checker = ProjectChecker(project_manager=project_manager)
project_checker.check_project()
