Feature: Users can read a specific Ao3 Work

Scenario: Opening a work displays work info
    Given a work blurb is onscreen
    When the user clicks on the work
    Then the work view is opened
    And the title is displayed
    And the author is displayed

Scenario: [BROKEN] Opening a large work displays loading
    Given a work blurb is onscreen
    And the work is large
    When the user clicks on the work
    Then a loading indicator is displayed
    And the work view is opened

Scenario: Works can be paged forwards
    Given a multi-page work is opened
    When the user clicks page forwards
    Then the page changes

Scenario: Works can be paged backwards
    Given a multi-page work is opened
    And the user has advanced a page
    When the user clicks page backwards
    Then the page changes

Scenario: Kudos can be added to a fic as guest
    Given a multi-page work is opened
    When the user clicks the kudos button
    Then an "Added Kudos" message is displayed

Scenario: Repeated kudos are not allowed
    Given a multi-page work is opened
    And a logged in user
    When the user clicks the kudos button
    Then an "Already added kudos" message is displayed

Scenario: Chapter Index jumps to correct page
    Given a multi-chapter work is opened
    When the user clicks on the index button
    And the user clicks on a future chapter
    Then the index is closed
    And the future chapter is displayed

Scenario: About Work opens work popup
    Given a multi-page work is opened
    When the user clicks About Work
    Then the work popup is opened
    And the work title is displayed
    And the work author is displayed
    And the Fandoms are displayed
    And the Tags are displayed
    And the Summary is displayed
