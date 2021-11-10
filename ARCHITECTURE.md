# Passerine's Architecture

> Note: this project is in the rapid initial stages of development, and as such, breaking changes or radical changes in project structure may occur. After the 1.0.0 release, this behavior will stabilize.

This document describes the high-level architecture of Passerine. This, along with the API documentation
and `CONTRIBUTING.md`, is the right place to get started.

Passerine strives to implement a modern compiler pipeline. Passerine is currently broken up into two small projects:

- The core compiler, which resides in this repository.
- The command line interface and the package repository, [Aspen](https://github.com/vrtbl/aspen).

This document details the implementation of Passerine's core, and does not discuss Aspen.

## View from Above

At the highest level, Passerine's compiler is just a series of pipes that transform one datatype into another. Each pipe
is responsible for one thing - all are located in `src/compiler`.

The final outline of the compiler pipeline is a `Lambda` - basically some bytecode packaged with the minimal amount of
context required to run it. This lambda can be instantiated by wrapping it with a `Closure`, then passed to the VM to
run it.

Passerine's VM is a fairly simple stack-based VM. you can find the implementation in `src/vm`. Sensibly, the core
interpreter loop is located in `src/vm/vm.rs`.

## Pipelines

## Pipe Data Structures

## Data and Core

## VM specifics
