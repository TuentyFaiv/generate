import SwiftUI

/// Groups atoms under one accessible label, such as a field with its hint.
struct ExampleMolecule<Content: View>: View {
    var label: LocalizedStringKey
    var description: LocalizedStringKey?
    @ViewBuilder var content: Content

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(label)
                .font(.subheadline.weight(.semibold))
            if let description {
                Text(description)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }
            HStack(spacing: 8) {
                content
            }
        }
        .accessibilityElement(children: .contain)
        .accessibilityLabel(Text(label))
    }
}

#Preview {
    ExampleMolecule(label: "Notifications", description: "Choose how we reach you.") {
        Button("Email") {}
        Button("Push") {}
    }
    .padding()
}
