# UNOler Rules

## Overview

This is a simple card game where players take turns playing cards. The goal of the game is to be the first player to empty their hand.

## Setup

* The game is played with a standard 108-card deck.
* The game is played with 7 cards per player.
* The top card of the deck is placed onto the discard pile.
* If the first card of the discard pile is a color-changing card, a random color is chosen.

## Turn Order

* Players take turns in a sequence.
* Play proceeds clockwise by default.
* Reverse cards flip the direction of play, clockwise becomes counter-clockwise, and vice versa.
* In two-player mode, reverse cards count as skip cards.

## Playing a Card

* A card is selected from the player's hand and put on the discard pile.
* A card can only be played if:
     * The card is the same color as the last card played.
     * The card has the same number as the last card played.
     * The card is the same type of special card as the last card played.
     * The card is a wild card.
* If you cannot (or choose not to) play a valid card, you may draw a card instead.

## Special Cards

* Draw Two:
     * The next player must draw two cards.
     * The card can be stacked with another draw two or a wild draw four.
* Wild:
     * The player chooses the color of the wild card.
* Wild Draw Four:
     * The player chooses the color of the wild card.
     * The next player must draw four cards.
     * This card **cannot** be countered.
* Skip:
     * The next player's turn is skipped, and they are unable to play a card.
* Reverse:
     * The direction of play is reversed.
     * In two-player mode, this card is a skip card.

## Stacking

* Draw Twos can stack.
* Wild Draw Fours cannot be countered.
* Stacking cards adds to a queue, and the player who cannot counter the drawing card is forced to draw the full queue.
* If a player has a card that can counter a draw two but chooses not to use it, they are forced to draw at the end of their turn.
* A Draw Four can stack onto a Draw Two, ending the stack accumulation.

## Winning

* When a player has one card left, "UNO" is declared.
* The first player to run out of cards wins.

