# Contributing to rulinalg

First of all thank you for your interest! I'm very keen to get more contributors onboard and am excited to help out in whichever ways I can.

Contributing can take place in many forms, including but not limited to:

- [Bug Reports](#bug-reports)
- [Feature Requests](#feature-requests)
- [Pull Requests](#pull-requests)

Bug Reports and Feature Requests are easy and the project is happily accepting them now. Please fire away!

As for Pull Requests I am excited to take on new contributors who can help with the code. Please see the section below about getting started.

---

## Bug Reports

If you're using rulinalg and run into what you believe to be a bug. Then please create an [issue](https://guides.github.com/features/issues/)
to let me know. Even if you're not confident this is a bug I'd prefer to hear about it!

In the [issue](https://guides.github.com/features/issues/) please include a description of the bug, and the conditions needed to replicate it.
Minimal conditions would be preferred but I understand this can often be a lot of work. If you can provide an example of the code producing
the bug this would be really handy too!

## Feature Requests

I strongly encourage feature requests! I'd love to get feedback and learn what the community wants to see next from this project.
This project is currently used within [rusty-machine](https://github.com/AtheMathmo/rusty-machine) and this will dictate much of
its development initially.

To request a feature please open an [issue](https://guides.github.com/features/issues/) with a description of the feature requested. If you can include some technical details and requirements this would be a big help.

## Pull Requests

This section will cover the process for making code contributions to rulinalg. Please feel free to make suggestions on how to improve this process (an issue on the repository will be fine).

### Getting Started

We currently use a [fork](https://help.github.com/articles/fork-a-repo/) and [pull request](https://help.github.com/articles/using-pull-requests/) model to allow contributions to rulinalg.

Please take a look through the code and [API documentation](https://athemathmo.github.io/rulinalg) to identify the areas you'd like to help out with. Take a look through the current
issues and see if there's anything you'd like to tackle. Simple issues will be tagged with the label `easy`.

If you decide you want to work on an issue please comment on that issue stating that you would like to work on it. This will help us keep track of who is working on what.
(I'm sure there's a better way to handle this - other ideas are welcome).

### Making Code Changes

So by now you should have the project forked and are ready to start working on the code. There are no hard conventions
in place at the moment but please follow these general guidelines:

- Document all public facing functions, structs, fields, etc. You can check this by adding `#![deny(missing_docs)]` to the top of the `lib.rs` file. This should include examples, panics and failures. (If you see these missing anywhere in the current code please create an issue.)
- Add comments to all private functions detailing what they do.
- Make lots of small commits as opposed to one large commit.
- Ensure that all existing (and new) tests pass for each commit.
- Add new tests for any new functionality you add. This means examples within the documentation, tests in the `tests` module, and integration tests in the _tests_ directory.
- Wherever possible add new benchmarks for new functionality or modifications to existing functionality. This area is currently lacking in rulinalg.
- There is (currently) no strict format for commit messages. But please be descriptive about the functionality you have added - this is much easier if using small commits as above!

### Creating the PR

Once the issue has been resolved please create the PR from your fork into the `master` branch. In the comments please reference the issue that the PR addresses, something like: "This resolves #XXX".

Other contributors will then review and give feedback. Once accepted the PR will be merged.
