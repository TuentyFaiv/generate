import SwiftUI

/// A distinct, labelled region of the screen, built from molecules.
struct ExampleOrganism<Content: View, Actions: View>: View {
    var heading: LocalizedStringKey
    @ViewBuilder var content: Content
    @ViewBuilder var actions: Actions

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack(alignment: .firstTextBaseline) {
                Text(heading)
                    .font(.title3.weight(.semibold))
                    .accessibilityAddTraits(.isHeader)
                Spacer()
                actions
            }
            content
        }
        .padding(24)
        .overlay {
            RoundedRectangle(cornerRadius: 12).stroke(.quaternary)
        }
        .accessibilityElement(children: .contain)
    }
}

extension ExampleOrganism where Actions == EmptyView {
    init(heading: LocalizedStringKey, @ViewBuilder content: () -> Content) {
        self.init(heading: heading, content: content, actions: { EmptyView() })
    }
}

#Preview {
    ExampleOrganism(heading: "Team") {
        Text("Three members")
    } actions: {
        Button("Invite") {}
    }
    .padding()
}
