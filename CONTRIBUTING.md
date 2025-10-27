Contributing to GeoPlegma
=========================

We welcome contributions to GeoPlegma and its associated code projects. These
can be in the form of issues, bug fixes, documentation or suggestions for
enhancements. This document sets out our guidelines and best practices for such
contributions.

## Code of Conduct

Contributors to this project are expected to act respectfully toward others in
accordance with the [Code of Conduct](CODE_OF_CONDUCT.md).

## Issues 

Any issue that is related to a implementation with a PR, should have the tag
`Feature`, `Task`, `Bug`. These tags are part of the default tag set from GitHub
with familiar meaning. These type of issues should also be related to a item on
the [kanban
board](https://github.com/GieoPlegma/GeoPlegma/projects?query=is%3Aopen). Other
matters that may not fit exactly with these familar tags should be tagged as
`Suggestion`.

## PR Protocol

Considering the growing complexity of this project, any pull request must be
well traceable and linked to existing procedures. In particular, it must address
a known issue and provide accompanying documentation. To make sure a pull
request adheres to these requirements please follow these steps:

1. The implementation needs to be associated with an item in the project [kanban
   board](https://github.com/GieoPlegma/GeoPlegma/projects?query=is%3Aopen).
2. Create a new branch and don't forget to always pull from `master`: 
``` 
git checkout master 
git checkout -b <name of the branch> 
git pull origin master
```
3. The name of the branch could have the initial context, which would be
   `feature`, `task`, `bug`, `refactor`, `hotfix`, something like:
   `feature/<branch_name>`
4. Make your changes and commit them.
5. Open PR. Add this checklist to the PR (I will create a template message so
   you dont need to add anything):
    - [ ] Link PR to the issue and kanban board item.
    - [ ] Write a list of what was done.
    - [ ] Add README documentation of what's done in the PR, if needed.
    - [ ] Request review
6. Wait for the review and any changes the reviewer(s) may require.
7. Squash and merge (never choose the other options, we dont want to join commit
history from the PR branch).

### Contributor Licence Agreement

Your contribution will be under the project licencing [licence](LICENCE.md) as
per [GitHub's terms of
service](https://help.github.com/articles/github-terms-of-service/#6-contributions-under-repository-license).



