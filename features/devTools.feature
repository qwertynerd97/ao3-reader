Feature: Developer tools

Scenario: Presing the screenshot button saves a screenshot
    Given the "screenshot_button" setting is set to "true"
    When the screenshot button is pressed
    Then a screenshot is saved
