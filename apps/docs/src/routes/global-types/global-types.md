# Global Types

To simplify the usage of types across your project, you can define them globally. This eliminates the need to pass type parameters everywhere (although that option is still available if preferred). Here's an example of how to set global types:

```typescript
declare global {
	interface EmberLink {
		Presence: {
			// Define the type for user presence data here
		};

		Custom: {
			// Define the type for custom messages here
		};
	}
}
```
