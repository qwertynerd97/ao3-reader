Feature: Users can browse a list of Ao3 works

Scenario: Opening a tag displays a list of works
    Given a multi-page tag is visible on screen
	When the user clicks on the tag
	Then a list of works is displayed

Scenario: Add tag to Favorites
    Given a multi-page tag is visible on screen
    And the multi-page tag is not in Favorites
	When the user clicks on the Favorite star
	Then a confirmation is shown
	And the tag is added to Favorites

Scenario: [BROKEN] Remove tag fom Favorites
    Given a multi-page tag is visible on screen
    And the multi-page tag is in Favorites
	When the user clicks on the Favorite star
	Then a confirmation is shown
	And the tag is removed from Favorites

Scenario: Multi-page tags can be paginated
    Given a multi-page tag is visible on screen
	When the user clicks on the tag
	And the user clicks through to the last page
	Then the last work in the tag is displayed

Scenario: [Broken] Short tags display list of works
    Given a single page tag is visible on screen
	When the user clicks on the tag
	Then a list of works is displayed

Scenario: About Work opens work popup
    Given a work list is open
    When the user long presses a work
    Then the work popup is opened
    And the work title is displayed
    And the work author is displayed
    And the Fandoms are displayed
    And the Tags are displayed
    And the Summary is displayed

Scenario: Long view displays tags
	Given the "work_display" setting is set to "Long"
	When a work list is viewed
	Then tags are displayed

Scenario: Short view hides tags
	Given the "work_display" setting is set to "Short"
	When a work list is viewed
	Then tags are hidden
