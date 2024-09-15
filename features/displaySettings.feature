Feature: Display Settings ajust how the Reader displays

Scenario Outline: Changing settings reloads the reader
    Given a multi-page work is opened
    And the Display Settings menu is open
    When the user clicks on <setting>
    And the user selects <value>
    Then the reader is reloaded
    And the new settings are used

    Examples:
        | setting      | value      |
        | Font Size    | 10.5       |
        | Text Align   | Right      |
        | Line Height  | 1.0        |
        | Margin Width | 3          |
        | Font Family  | Comic Sans |

Scenario: Landscape orientation can be used
    Given the device is in portrait mode
    When the user swaps to Landscape
    Then the reader is reloaded
