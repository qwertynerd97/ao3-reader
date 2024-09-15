Feature: The home view enables access to primary user actions

Scenario: [BROKEN] Wifi is force-enabled
    When the user visits the home screen
    Then wifi should be connected

Scenario: [HALF-BROKEN] The user can in to their Ao3 account
    Given no user is logged in
    When the user enters their user name
    And the user enters their password
    And the user clicks log in
    Then the login information is saved
    And the user is logged in
    And a log out button is displayed

Scenario: [BROKEN] The user can log out of their Ao3 account
    Given a logged in user
    When the user clicks log out
    Then the login info is cleared
    And the login button is displayed

Scenario: Favorite work lists are displayed
    Given works lists have been favorited
    When the user visits the home page
    Then the favorite lists are displayed

Scenario: Logged in users can access their Marked For Later
    Given a logged in user
    When the user visits the home page
    Then the Marked For Later list is visible
