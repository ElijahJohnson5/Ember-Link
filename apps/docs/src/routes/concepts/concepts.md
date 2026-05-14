# **Concepts**

Ember Link is built out of five primitives. Every feature described in the rest of these docs is some combination of them. This page is a one-paragraph tour of each, with links to the dedicated page for the details.

## **[Client](/concepts/client)**

The Client is the SDK's entry point. Each application creates one Client, which owns the WebSocket connection and the authentication lifecycle. When you ask the Client to join a channel by name, it ref-counts the channel internally, so two unrelated components that both join the same channel share one underlying socket.

## **[Channels](/concepts/channels)**

A Channel is a named room on the server. Every client that joins the same channel name on the same server can see the other clients in that channel and share its storage with them. Channels are designed to be cheap, so you can create as many of them as your application needs.

## **[Presence](/concepts/presence)**

Presence is per-user, ephemeral state, such as a cursor position, a typing indicator, or a selected color. Presence is removed automatically when the user disconnects from the channel. The SDK supports coalescing presence sends through the `presenceThrottle` channel option, so applications that update presence at the screen refresh rate can still keep their on-wire send rate reasonable.

## **[Storage](/concepts/storage)**

Storage is a CRDT-backed document shared across every client in the channel. Updates from any client are merged conflict-free with updates from any other client, and the merged state is persisted on the server. Storage is the right place for any data that should outlive the session, such as the body of a document, the items in a todo list, or the shapes on a canvas.

## **[Custom messages](/concepts/custom-messages)**

Custom messages are a typed pub/sub layer between clients in a channel. The SDK delivers them fire-and-forget to every connected peer. Custom messages are a good fit for ephemeral events that do not fit cleanly into presence or storage, such as a "user X just clicked the buzzer" notification, or a chat message that your own backend is about to persist.

## **[Type augmentation](/concepts/type-augmentation)**

In addition to the five primitives above, Ember Link ships a TypeScript ergonomics feature called type augmentation. Type augmentation lets you declare the shape of your `Presence` and `Custom` types once, globally, so that every API across every SDK picks up those types automatically without you having to thread generic type parameters through every provider, hook, and channel call.
